use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tracing::info;

use crate::config::transport::tcp::{AcceptorConfig, ConnectorConfig};

#[derive(Debug)]
pub struct Acceptor {
    listener: TcpListener,
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Self, anyhow::Error> {
        info!("Listening on {}", config.listen);
        Ok(Self {
            listener: TcpListener::bind(config.listen).await?,
        })
    }

    pub async fn accept(&self) -> Result<TcpStream, anyhow::Error> {
        let (s, _) = self.listener.accept().await?;
        Ok(s)
    }
}

#[derive(Debug)]
pub struct Connector {
    endpoint: SocketAddr,
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            endpoint: config.endpoint,
        })
    }

    pub async fn connect(&self) -> Result<TcpStream, anyhow::Error> {
        Ok(TcpStream::connect(&self.endpoint).await?)
    }
}
