use std::{pin::Pin, str::FromStr};

use anyhow::{bail, Context};
use bytes::Bytes;
use futures::Future;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body, http, server::conn::http1, service::Service, Method, Request, Response, StatusCode,
};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, debug_span, error, instrument, warn, Instrument};

use crate::{
    config::{egress::http::ClientConfig, ingress::http::ServerConfig},
    io::BoxedAsyncIO,
    net::transport,
    proxy::{Address, ForwardHttpConn, ForwardTcpConn, NetLocation, ProxyConn, ProxyRequest},
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
            .await
            .context("")?;
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
                conn: ProxyConn::ForwardHttp(ForwardHttpConn { req, resp_tx: tx }),
            })
            .unwrap();

        rx.await.context("Failed to get response")
    }

    async fn handle_https_proxy(
        self,
        req: Request<body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
        match host_addr(req.uri()) {
            Ok(remote) => {
                tokio::task::spawn(
                    async move {
                        match hyper::upgrade::on(req).await {
                            Ok(upgraded) => self
                                .tx
                                .send(ProxyRequest {
                                    remote,
                                    conn: ProxyConn::ForwardTcp(ForwardTcpConn {
                                        stream: Box::new(upgraded),
                                    }),
                                })
                                .unwrap(),
                            Err(e) => error!("upgrade error: {:?}", e),
                        }
                    }
                    .instrument(debug_span!("CONNECT")),
                );

                Ok(Response::new(empty()))
            }
            Err(e) => {
                warn!("CONNECT host is not socket addr {:?}: {}", req.uri(), e);

                let mut resp = Response::new(full("CONNECT must be to a socket address"));
                *resp.status_mut() = http::StatusCode::BAD_REQUEST;
                Ok(resp)
            }
        }
    }
}

impl Service<Request<hyper::body::Incoming>> for ServerService {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;

    type Error = anyhow::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[instrument(
        name = "handle_request",
        skip_all,
        fields(http.method = req.method().to_string(),
               http.uri = req.uri().to_string())
    )]
    fn call(&mut self, req: Request<hyper::body::Incoming>) -> Self::Future {
        let self_ = self.clone();
        Box::pin(async move {
            debug!(monotonic_counter.http_proxy_request = 1);

            if Method::CONNECT == req.method() {
                ServerService::handle_https_proxy(self_, req).await
            } else if req.headers().contains_key("Proxy-Connection") {
                ServerService::handle_http_proxy(self_, req).await
            } else {
                ServerService::handle_http(req).await
            }
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
    ) -> Result<(), anyhow::Error> {
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

        debug!("{:?}", res);

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
        forward_conn: ForwardHttpConn,
    ) -> Result<(), anyhow::Error> {
        forward_conn.forward(s).await
    }

    pub async fn handle_proxy_request(&self, req: ProxyRequest) -> Result<(), anyhow::Error> {
        let s = self
            .connector
            .connect()
            .await
            .context("Failed to connect")?;

        match req.conn {
            ProxyConn::ForwardTcp(conn) => self.handle_forward_tcp(s, req.remote, conn).await,
            ProxyConn::ForwardHttp(conn) => self.handle_forward_http(s, conn).await,
        }
    }
}
