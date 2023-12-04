use std::net::SocketAddr;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tracing::{info, instrument};

use crate::{config::transport::tcp::{AcceptorConfig, ConnectorConfig}, net::tool::dynamic_port};

use super::Connect;

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
    inner: dynamic_port::Connector<ConnectorInner, TcpStream>,
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: dynamic_port::Connector::new(ConnectorInner::new(), config.endpoint).await?,
        })
    }

    #[instrument(skip_all, fields(transport = "tcp"))]
    pub async fn connect(&self) -> Result<TcpStream, anyhow::Error> {
        self.inner.connect().await
    }
}

#[derive(Debug)]
struct ConnectorInner {
    endpoint: RwLock<Option<SocketAddr>>,
}

impl ConnectorInner {
    fn new() -> Self {
        Self {
            endpoint: RwLock::new(None),
        }
    }
}

impl Connect<TcpStream> for ConnectorInner {
    async fn is_open(&self) -> bool {
        self.endpoint.read().await.is_some()
    }

    async fn connect(&self, endpoint: SocketAddr) -> Result<(), anyhow::Error> {
        *self.endpoint.write().await = Some(endpoint);
        Ok(())
    }

    async fn open_stream(&self) -> Result<TcpStream, anyhow::Error> {
        if let Some(endpoint) = *self.endpoint.read().await {
            Ok(TcpStream::connect(endpoint).await?)
        } else {
            anyhow::bail!("connection is invalid")
        }
    }

    async fn close(&self) {
        *self.endpoint.write().await = None;
    }
}
