mod dynamic_port;
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
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Acceptor, anyhow::Error> {
        Ok(match config {
            AcceptorConfig::Quic(config) => Acceptor::Quic(quic::Acceptor::new(config).await?),
            AcceptorConfig::Tcp(config) => Acceptor::Tcp(tcp::Acceptor::new(config).await?),
        })
    }

    pub async fn accept(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Acceptor::Quic(acceptor) => Box::new(acceptor.accept().await?),
            Acceptor::Tcp(acceptor) => Box::new(acceptor.accept().await?),
        })
    }
}

#[derive(Debug)]
pub enum Connector {
    Quic(quic::Connector),
    Tcp(tcp::Connector),
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Connector, anyhow::Error> {
        Ok(match config {
            ConnectorConfig::Quic(config) => Connector::Quic(quic::Connector::new(config).await?),
            ConnectorConfig::Tcp(config) => Connector::Tcp(tcp::Connector::new(config).await?),
        })
    }

    pub async fn connect(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Connector::Quic(connector) => Box::new(connector.connect().await?),
            Connector::Tcp(connector) => Box::new(connector.connect().await?),
        })
    }
}

pub trait Connect<S> {
    async fn connect(&self, endpoint: SocketAddr) -> Result<(), anyhow::Error>;
    async fn open_stream(&self) -> Result<S, anyhow::Error>;
}
