mod dynamic_port;
mod kcp;
mod quic;
mod tcp;

use std::net::SocketAddr;

use crate::{
    config::transport::{AcceptorConfig, ConnectorConfig},
    io::BoxedAsyncIO,
};

#[derive(Debug)]
pub enum Acceptor {
    Quic(quic::Acceptor),
    Tcp(tcp::Acceptor),
    Kcp(kcp::Acceptor),
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Acceptor, anyhow::Error> {
        Ok(match config {
            AcceptorConfig::Quic(config) => Acceptor::Quic(quic::Acceptor::new(config).await?),
            AcceptorConfig::Tcp(config) => Acceptor::Tcp(tcp::Acceptor::new(config).await?),
            AcceptorConfig::Kcp(config) => Acceptor::Kcp(kcp::Acceptor::new(config).await?),
        })
    }

    pub async fn accept(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Acceptor::Quic(acceptor) => Box::new(acceptor.accept().await?) as BoxedAsyncIO,
            Acceptor::Tcp(acceptor) => Box::new(acceptor.accept().await?),
            Acceptor::Kcp(acceptor) => Box::new(acceptor.accept().await?),
        })
    }
}

#[derive(Debug)]
pub enum Connector {
    Quic(quic::Connector),
    Tcp(tcp::Connector),
    Kcp(kcp::Connector),
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Connector, anyhow::Error> {
        Ok(match config {
            ConnectorConfig::Quic(config) => Connector::Quic(quic::Connector::new(config).await?),
            ConnectorConfig::Tcp(config) => Connector::Tcp(tcp::Connector::new(config).await?),
            ConnectorConfig::Kcp(config) => Connector::Kcp(kcp::Connector::new(config).await?),
        })
    }

    pub async fn connect(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Connector::Quic(connector) => Box::new(connector.connect().await?) as BoxedAsyncIO,
            Connector::Tcp(connector) => Box::new(connector.connect().await?),
            Connector::Kcp(connector) => Box::new(connector.connect().await?),
        })
    }
}

pub trait Connect<S> {
    async fn is_open(&self) -> bool;
    async fn connect(&self, endpoint: SocketAddr) -> Result<(), anyhow::Error>;
    async fn close(&self);
    async fn open_stream(&self) -> Result<S, anyhow::Error>;
}
