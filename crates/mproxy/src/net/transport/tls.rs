use std::{sync::Arc, time::Duration};

use rustls_pki_types::ServerName;
use tokio::time::timeout;
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

    pub async fn accept(&self) -> Result<BoxedAsyncIO, anyhow::Error> {
        Ok(self.next_layer.accept().await?)
    }

    pub async fn handshake(
        &self,
        io: BoxedAsyncIO,
    ) -> Result<server::TlsStream<BoxedAsyncIO>, anyhow::Error> {
        Ok(self.acceptor.accept(io).await?)
    }
}

pub struct Connector {
    next_layer: super::Connector,
    connector: TlsConnector,
    server_name: String,

    handshake_timeout: Duration,
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
    pub async fn new(
        ConnectorConfig {
            next_layer,
            tls,
            server_name,
            handshake_timeout,
        }: ConnectorConfig,
    ) -> Result<Self, anyhow::Error> {
        let mut tls_config = rustls::ClientConfig::try_from(&tls)?;

        tls_config.resumption = rustls::client::Resumption::in_memory_sessions(128);

        Ok(Self {
            next_layer: super::Connector::new(next_layer).await?,
            connector: TlsConnector::from(Arc::new(tls_config)),
            server_name,
            handshake_timeout,
        })
    }

    #[instrument(skip_all, fields(transport = "tls"))]
    pub async fn connect(&self) -> Result<client::TlsStream<BoxedAsyncIO>, anyhow::Error> {
        Ok(timeout(
            self.handshake_timeout.clone(),
            self.connector.connect(
                ServerName::try_from(self.server_name.clone())?,
                self.next_layer.connect().await?,
            ),
        )
        .await??)
    }
}
