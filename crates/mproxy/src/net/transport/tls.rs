use std::sync::Arc;

use tokio_rustls::{client, server, TlsAcceptor, TlsConnector};
use tracing::instrument;

use crate::{
    config::transport::tls::{AcceptorConfig, ConnectorConfig},
    io::BoxedAsyncIO,
};

pub struct Acceptor {
    next_layer: super::Acceptor,
    acceptor: TlsAcceptor,
}

impl std::fmt::Debug for Acceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acceptor")
            .field("next_layer", &self.next_layer)
            .finish()
    }
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Self, anyhow::Error> {
        let tls_config = rustls::ServerConfig::try_from(&config.tls)?;

        Ok(Self {
            next_layer: super::Acceptor::new(config.next_layer).await?,
            acceptor: TlsAcceptor::from(Arc::new(tls_config)),
        })
    }

    pub async fn accept(&self) -> Result<server::TlsStream<BoxedAsyncIO>, anyhow::Error> {
        Ok(self
            .acceptor
            .accept(self.next_layer.accept().await?)
            .await?)
    }
}

pub struct Connector {
    next_layer: super::Connector,
    connector: TlsConnector,
    server_name: String,
}

impl std::fmt::Debug for Connector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connector")
            .field("next_layer", &self.next_layer)
            .field("server_name", &self.server_name)
            .finish()
    }
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        let tls_config = rustls::ClientConfig::try_from(&config.tls)?;
        Ok(Self {
            next_layer: super::Connector::new(config.next_layer).await?,
            connector: TlsConnector::from(Arc::new(tls_config)),
            server_name: config.server_name,
        })
    }

    #[instrument(skip_all, fields(transport = "tls"))]
    pub async fn connect(&self) -> Result<client::TlsStream<BoxedAsyncIO>, anyhow::Error> {
        Ok(self
            .connector
            .connect(
                rustls::ServerName::try_from(self.server_name.as_str())?,
                self.next_layer.connect().await?,
            )
            .await?)
    }
}
