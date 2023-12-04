use std::net::SocketAddr;

use tokio::sync::{Mutex, RwLock};
use tokio_kcp::{KcpConfig, KcpListener, KcpStream};
use tracing::{debug, info, instrument};

use crate::{config::transport::kcp::{AcceptorConfig, ConnectorConfig}, net::tool::dynamic_port};

use super::Connect;

#[derive(Debug)]
pub struct Acceptor {
    listener: Mutex<KcpListener>,
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Self, anyhow::Error> {
        info!("Listening on {}", config.listen);
        Ok(Self {
            listener: Mutex::new(KcpListener::bind(config.kcp, config.listen).await?),
        })
    }

    pub async fn accept(&self) -> Result<KcpStream, anyhow::Error> {
        let (s, _) = self.listener.lock().await.accept().await?;
        Ok(s)
    }
}

#[derive(Debug)]
pub struct Connector {
    inner: dynamic_port::Connector<ConnectorInner, KcpStream>,
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        debug!("{:?}", config);

        Ok(Self {
            inner: dynamic_port::Connector::new(ConnectorInner::new(config.kcp), config.endpoint)
                .await?,
        })
    }

    #[instrument(skip_all, fields(transport = "kcp"))]
    pub async fn connect(&self) -> Result<KcpStream, anyhow::Error> {
        self.inner.connect().await
    }
}

#[derive(Debug)]
struct ConnectorInner {
    config: KcpConfig,
    endpoint: RwLock<Option<SocketAddr>>,
}

impl ConnectorInner {
    fn new(config: KcpConfig) -> Self {
        Self {
            config,
            endpoint: RwLock::new(None),
        }
    }
}

impl Connect<KcpStream> for ConnectorInner {
    async fn is_open(&self) -> bool {
        self.endpoint.read().await.is_some()
    }

    async fn connect(&self, endpoint: SocketAddr) -> Result<(), anyhow::Error> {
        *self.endpoint.write().await = Some(endpoint);
        Ok(())
    }

    async fn open_stream(&self) -> Result<KcpStream, anyhow::Error> {
        if let Some(endpoint) = *self.endpoint.read().await {
            Ok(KcpStream::connect(&self.config, endpoint).await?)
        } else {
            anyhow::bail!("connection is invalid")
        }
    }

    async fn close(&self) {
        *self.endpoint.write().await = None;
    }
}
