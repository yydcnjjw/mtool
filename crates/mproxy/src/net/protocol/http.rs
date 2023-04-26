use std::{pin::Pin, str::FromStr};

use anyhow::{bail, Context};
use futures::Future;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::{self, Bytes},
    http::{self, uri::Scheme},
    server::conn::http1,
    service::Service,
    Method, Request, Response, StatusCode,
};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, debug_span, error, instrument, warn, Instrument};

use crate::{
    config::{egress::http::ClientConfig, ingress::http::ServerConfig},
    io::BoxedAsyncIO,
    net::transport,
    proxy::{
        Address, ForwardHttpConn, ForwardTcpConn, NetLocation, ProxyConn, ProxyRequest,
        ProxyResponse,
    },
};

#[derive(Debug)]
pub struct Server {
    acceptor: transport::Acceptor,
    tx: mpsc::UnboundedSender<ProxyRequest>,
    rx: Mutex<mpsc::UnboundedReceiver<ProxyRequest>>,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            acceptor: transport::Acceptor::new(config.acceptor).await?,
            tx,
            rx: Mutex::new(rx),
        })
    }

    async fn serve_inner(
        tx: mpsc::UnboundedSender<ProxyRequest>,
        stream: BoxedAsyncIO,
    ) -> Result<(), anyhow::Error> {
        http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .serve_connection(stream, ServerService { tx })
            .with_upgrades()
            .await?;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn serve(tx: mpsc::UnboundedSender<ProxyRequest>, stream: BoxedAsyncIO) {
        if let Err(e) = Self::serve_inner(tx, stream).await {
            warn!("{:?}", e);
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        loop {
            let stream = self.acceptor.accept().await?;
            let tx = self.tx.clone();
            tokio::task::spawn(Self::serve(tx, stream));
        }
    }

    pub async fn proxy_accept(&self) -> Result<ProxyRequest, anyhow::Error> {
        let mut rx = self.rx.lock().await;

        rx.recv().await.context("Failed to proxy accept")
    }
}

#[derive(Clone)]
struct ServerService {
    tx: mpsc::UnboundedSender<ProxyRequest>,
}

impl ServerService {
    async fn handle_http(
        _req: Request<body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
        let mut resp = Response::new(full("The request is not supported"));
        *resp.status_mut() = http::StatusCode::BAD_REQUEST;
        Ok(resp)
    }

    async fn handle_http_proxy(
        self,
        req: Request<body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
        let remote = host_addr(req.uri())?;
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ProxyRequest {
                remote,
                conn: ProxyConn::ForwardHttp(ForwardHttpConn::new(req, tx)),
            })
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        rx.await.context("Failed to get response")?
    }

    async fn handle_https_proxy(
        self,
        req: Request<body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
        let remote = host_addr(req.uri()).context("socket address is incorrect at CONNECT")?;

        tokio::task::spawn(
            async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = self.tx.send(ProxyRequest {
                            remote,
                            conn: ProxyConn::ForwardTcp(ForwardTcpConn {
                                stream: Box::new(upgraded),
                            }),
                        }) {
                            warn!("{:?}", e);
                        }
                    }
                    Err(e) => error!("upgrade error: {:?}", e),
                }
            }
            .instrument(debug_span!("CONNECT")),
        );

        Ok(Response::new(empty()))
    }

    fn is_https_proxy_request(req: &Request<hyper::body::Incoming>) -> bool {
        req.method() == Method::CONNECT
    }

    fn is_http_proxy_request(req: &Request<hyper::body::Incoming>) -> bool {
        req.headers().contains_key("proxy-connection")
            || req
                .uri()
                .scheme()
                .and_then(|v| if v == &Scheme::HTTP { Some(v) } else { None })
                .is_some()
    }

    #[instrument(
        name = "handle_request",
        skip_all,
        fields(http.method = req.method().to_string(),
               http.uri = req.uri().to_string())
    )]
    async fn handle_request(
        self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
        debug!(monotonic_counter.http_proxy_request = 1);

        if Self::is_https_proxy_request(&req) {
            self.handle_https_proxy(req).await
        } else if Self::is_http_proxy_request(&req) {
            self.handle_http_proxy(req).await
        } else {
            Self::handle_http(req).await
        }
    }
}

impl Service<Request<hyper::body::Incoming>> for ServerService {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;

    type Error = anyhow::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<hyper::body::Incoming>) -> Self::Future {
        let self_ = self.clone();
        Box::pin(async move {
            self_.handle_request(req).await.or_else(|e| {
                let mut resp = Response::new(full(format!("{:?}", e)));
                *resp.status_mut() = http::StatusCode::BAD_REQUEST;
                Ok(resp)
            })
        })
    }
}

fn host_addr(uri: &http::Uri) -> Result<NetLocation, anyhow::Error> {
    let address = Address::from_str(uri.host().context("host isn't exist")?)?;
    Ok(NetLocation {
        address,
        port: uri.port_u16().unwrap_or(80),
    })
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[derive(Debug)]
pub struct Client {
    connector: transport::Connector,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            connector: transport::Connector::new(config.connector).await?,
        })
    }

    async fn handle_forward_tcp(
        &self,
        s: BoxedAsyncIO,
        remote: NetLocation,
        forward_conn: ForwardTcpConn,
    ) -> Result<(u64, u64), anyhow::Error> {
        let (mut sender, conn) = hyper::client::conn::http1::handshake(s).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection failed: {:?}", err);
            }
        });

        let req = Request::builder()
            .method(Method::CONNECT)
            .uri(remote.to_string())
            .body(Empty::<Bytes>::new())?;

        let res = sender.send_request(req).await?;

        if res.status() != StatusCode::OK {
            bail!(
                "{}",
                std::str::from_utf8(&res.collect().await?.to_bytes()[..])?
            )
        }

        forward_conn.forward(hyper::upgrade::on(res).await?).await
    }

    async fn handle_forward_http(
        &self,
        s: BoxedAsyncIO,
        mut forward_conn: ForwardHttpConn,
    ) -> Result<(u64, u64), anyhow::Error> {
        forward_conn.remove_proxy_header = false;
        forward_conn.forward(s).await
    }

    pub async fn send(&self, req: ProxyRequest) -> Result<ProxyResponse, anyhow::Error> {
        let s = self
            .connector
            .connect()
            .await
            .context("Failed to connect")?;

        let (upload_bytes, download_bytes) = match req.conn {
            ProxyConn::ForwardTcp(conn) => self.handle_forward_tcp(s, req.remote, conn).await?,
            ProxyConn::ForwardHttp(conn) => self.handle_forward_http(s, conn).await?,
        };

        Ok(ProxyResponse {
            upload_bytes,
            download_bytes,
        })
    }
}
