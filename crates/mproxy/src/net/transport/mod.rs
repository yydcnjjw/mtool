mod quic;
mod tcp;

use core::fmt;

use tracing::instrument;

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

impl fmt::Display for Connector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Connector::Quic(_) => write!(f, "quic"),
            Connector::Tcp(_) => write!(f, "tcp"),
        }
    }
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Connector, anyhow::Error> {
        Ok(match config {
            ConnectorConfig::Quic(config) => Connector::Quic(quic::Connector::new(config).await?),
            ConnectorConfig::Tcp(config) => Connector::Tcp(tcp::Connector::new(config).await?),
        })
    }

    #[instrument(skip_all, fields(transport = self.to_string()))]
    pub async fn connect(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(match self {
            Self::Quic(connector) => Box::new(connector.connect().await?),
            Self::Tcp(connector) => Box::new(connector.connect().await?),
        })
    }
}
