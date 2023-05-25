use std::sync::Arc;

use anyhow::Context;
use socksv5::{
    v4::SocksV4Command,
    v5::{SocksV5AuthMethod, SocksV5Command, SocksV5RequestStatus},
    SocksVersion,
};
use tokio::sync::{mpsc, Mutex};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};
use tracing::{instrument, warn};

use crate::{
    config::ingress::socks::{ServerConfig, Socks5Config},
    io::BoxedAsyncIO,
    net::transport,
    proxy::{Address, NetLocation, ProxyConn, ProxyRequest, TcpForwarder},
};

#[derive(Debug)]
pub struct Server {
    acceptor: transport::Acceptor,
    config: Arc<Socks5Config>,

    tx: mpsc::UnboundedSender<ProxyRequest>,
    rx: Mutex<mpsc::UnboundedReceiver<ProxyRequest>>,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            acceptor: transport::Acceptor::new(config.acceptor).await?,
            config: Arc::new(config.socks5),
            tx,
            rx: Mutex::new(rx),
        })
    }

    async fn serve_socksv5(
        tx: mpsc::UnboundedSender<ProxyRequest>,
        mut stream: Compat<BoxedAsyncIO>,
    ) -> Result<(), anyhow::Error> {
        let _methods = socksv5::v5::read_handshake_skip_version(&mut stream).await?;

        // if let Some(_auth) = &config.auth {
        //     anyhow::bail!("auth is not supported {:?}", methods)
        // }

        socksv5::v5::write_auth_method(&mut stream, SocksV5AuthMethod::Noauth).await?;

        let request = socksv5::v5::read_request(&mut stream).await?;

        let request = match request.command {
            SocksV5Command::Connect | SocksV5Command::Bind => {
                socksv5::v5::write_request_status(
                    &mut stream,
                    SocksV5RequestStatus::Success,
                    socksv5::v5::SocksV5Host::Ipv4([0, 0, 0, 0]),
                    0,
                )
                .await?;

                ProxyRequest {
                    remote: NetLocation {
                        address: Address::try_from(request.host)?,
                        port: request.port,
                    },
                    conn: ProxyConn::ForwardTcp(TcpForwarder {
                        stream: stream.into_inner(),
                    }),
                }
            }
            _ => {
                socksv5::v5::write_request_status(
                    &mut stream,
                    SocksV5RequestStatus::CommandNotSupported,
                    socksv5::v5::SocksV5Host::Ipv4([0, 0, 0, 0]),
                    0,
                )
                .await?;
                anyhow::bail!("{:?} is not supported", request.command)
            }
        };
        tx.send(request)
            .map_err(|e| anyhow::anyhow!("send error: {:?}", e.0))
    }

    async fn serve_socksv4(
        tx: mpsc::UnboundedSender<ProxyRequest>,
        mut stream: Compat<BoxedAsyncIO>,
    ) -> Result<(), anyhow::Error> {
        let request = socksv5::v4::read_request_skip_version(&mut stream).await?;
        match request.command {
            SocksV4Command::Connect | SocksV4Command::Bind => {
                socksv5::v4::write_request_status(
                    &mut stream,
                    socksv5::v4::SocksV4RequestStatus::Granted,
                    [0, 0, 0, 0],
                    0,
                )
                .await?
            }
        }

        tx.send(ProxyRequest {
            remote: NetLocation {
                address: Address::try_from(request.host)?,
                port: request.port,
            },
            conn: ProxyConn::ForwardTcp(TcpForwarder {
                stream: stream.into_inner(),
            }),
        })
        .map_err(|e| anyhow::anyhow!("send error: {:?}", e.0))
    }

    async fn serve_inner(
        tx: mpsc::UnboundedSender<ProxyRequest>,
        stream: BoxedAsyncIO,
        _config: Arc<Socks5Config>,
    ) -> Result<(), anyhow::Error> {
        let mut stream = stream.compat();

        match socksv5::read_version(&mut stream).await? {
            SocksVersion::V4 => Self::serve_socksv4(tx, stream).await,
            SocksVersion::V5 => Self::serve_socksv5(tx, stream).await,
        }
    }

    #[instrument(skip_all)]
    async fn serve(
        tx: mpsc::UnboundedSender<ProxyRequest>,
        stream: BoxedAsyncIO,
        config: Arc<Socks5Config>,
    ) {
        if let Err(e) = Self::serve_inner(tx, stream, config).await {
            warn!("{:?}", e);
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        loop {
            let stream = self.acceptor.accept().await?;
            tokio::task::spawn(Self::serve(self.tx.clone(), stream, self.config.clone()));
        }
    }

    pub async fn proxy_accept(&self) -> Result<ProxyRequest, anyhow::Error> {
        let mut rx = self.rx.lock().await;

        rx.recv().await.context("Failed to proxy accept")
    }
}