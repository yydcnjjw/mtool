use std::sync::Arc;

use socksv5::{
    v4::SocksV4Command,
    v5::{SocksV5AuthMethod, SocksV5Command, SocksV5RequestStatus},
    SocksVersion,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
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
    acceptor: Arc<transport::Acceptor>,
    config: Arc<Socks5Config>,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            acceptor: Arc::new(transport::Acceptor::new(config.acceptor).await?),
            config: Arc::new(config.socks5),
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
            SocksV5Command::UdpAssociate => {
                socksv5::v5::write_request_status(
                    &mut stream,
                    SocksV5RequestStatus::CommandNotSupported,
                    socksv5::v5::SocksV5Host::Ipv4([0, 0, 0, 0]),
                    0,
                )
                .await?;
                anyhow::bail!("{:?} is not supported", request)
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

    pub async fn incoming(&self) -> Result<UnboundedReceiverStream<ProxyRequest>, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(Self::run(self.acceptor.clone(), tx, self.config.clone()));

        Ok(UnboundedReceiverStream::new(rx))
    }

    async fn run(
        acceptor: Arc<transport::Acceptor>,
        tx: mpsc::UnboundedSender<ProxyRequest>,
        config: Arc<Socks5Config>,
    ) {
        loop {
            match acceptor.accept().await {
                Ok(stream) => tokio::spawn(Self::serve(tx.clone(), stream, config.clone())),
                Err(e) => {
                    warn!("{:?}", e);
                    break;
                }
            };
        }
    }
}
