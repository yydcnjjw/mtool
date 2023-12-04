mod kcp;
mod quic;
mod tcp;
mod tls;

use std::net::SocketAddr;

use async_recursion::async_recursion;

use crate::{
    config::transport::{AcceptorConfig, ConnectorConfig, ConnectorConfigInner},
    io::{BoxedAsyncIO, TimeoutStream},
};

#[derive(Debug)]
pub enum Acceptor {
    Quic(quic::Acceptor),
    Tcp(tcp::Acceptor),
    Kcp(kcp::Acceptor),
    Tls(Box<tls::Acceptor>),
}

impl Acceptor {
    #[async_recursion]
    pub async fn new(config: AcceptorConfig) -> Result<Acceptor, anyhow::Error> {
        Ok(match config {
            AcceptorConfig::Quic(config) => Acceptor::Quic(quic::Acceptor::new(config).await?),
            AcceptorConfig::Tcp(config) => Acceptor::Tcp(tcp::Acceptor::new(config).await?),
            AcceptorConfig::Kcp(config) => Acceptor::Kcp(kcp::Acceptor::new(config).await?),
            AcceptorConfig::Tls(config) => {
                Acceptor::Tls(Box::new(tls::Acceptor::new(*config).await?))
            }
        })
    }

    #[async_recursion]
    pub async fn accept(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Acceptor::Quic(acceptor) => Box::new(acceptor.accept().await?) as BoxedAsyncIO,
            Acceptor::Tcp(acceptor) => Box::new(acceptor.accept().await?),
            Acceptor::Kcp(acceptor) => Box::new(acceptor.accept().await?),
            Acceptor::Tls(acceptor) => Box::new(acceptor.accept().await?),
        })
    }

    #[async_recursion]
    pub async fn handshake(&self, io: BoxedAsyncIO) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Acceptor::Tls(acceptor) => Box::new(acceptor.handshake(io).await?) as BoxedAsyncIO,
            _ => io,
        })
    }
}

#[derive(Debug)]
pub enum ConnectorInner {
    Quic(quic::Connector),
    Tcp(tcp::Connector),
    Kcp(kcp::Connector),
    Tls(Box<tls::Connector>),
}

#[derive(Debug)]
pub struct Connector {
    inner: ConnectorInner,
    config: ConnectorConfig,
}

impl Connector {
    #[async_recursion]
    pub async fn new(config: ConnectorConfig) -> Result<Connector, anyhow::Error> {
        Ok(Self {
            inner: match config.inner.clone() {
                ConnectorConfigInner::Quic(config) => {
                    ConnectorInner::Quic(quic::Connector::new(config).await?)
                }
                ConnectorConfigInner::Tcp(config) => {
                    ConnectorInner::Tcp(tcp::Connector::new(config).await?)
                }
                ConnectorConfigInner::Kcp(config) => {
                    ConnectorInner::Kcp(kcp::Connector::new(config).await?)
                }
                ConnectorConfigInner::Tls(config) => {
                    ConnectorInner::Tls(Box::new(tls::Connector::new(*config).await?))
                }
            },
            config,
        })
    }

    #[async_recursion]
    pub async fn connect(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        let io = match &self.inner {
            ConnectorInner::Quic(connector) => Box::new(connector.connect().await?) as BoxedAsyncIO,
            ConnectorInner::Tcp(connector) => Box::new(connector.connect().await?),
            ConnectorInner::Kcp(connector) => Box::new(connector.connect().await?),
            ConnectorInner::Tls(connector) => Box::new(connector.connect().await?),
        };

        let mut io = TimeoutStream::new(io);
        io.set_read_timeout(Some(self.config.transport.read_timeout));
        io.set_write_timeout(Some(self.config.transport.write_timeout));
        Ok(Box::new(io) as BoxedAsyncIO)
    }
}

pub trait Connect<S> {
    async fn is_open(&self) -> bool;
    async fn connect(&self, endpoint: SocketAddr) -> Result<(), anyhow::Error>;
    async fn close(&self);
    async fn open_stream(&self) -> Result<S, anyhow::Error>;
}
