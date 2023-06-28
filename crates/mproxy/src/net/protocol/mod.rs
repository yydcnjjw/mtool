pub mod direct;
pub mod http;
pub mod socks;

use core::fmt;

use futures::Stream;
use tracing::instrument;

use crate::{
    config::{egress::ClientConfig, ingress::ServerConfig},
    proxy::{ProxyRequest, ProxyResponse},
};

#[derive(Debug)]
pub enum Server {
    Http(http::Server),
    Socks(socks::Server),
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self, anyhow::Error> {
        Ok(match config {
            ServerConfig::Http(config) => Self::Http(http::Server::new(config).await?),
            ServerConfig::Socks(config) => Self::Socks(socks::Server::new(config).await?),
        })
    }

    #[instrument(skip_all)]
    pub async fn incoming(
        &self,
    ) -> Result<Box<dyn Stream<Item = ProxyRequest> + Unpin + Send>, anyhow::Error> {
        Ok(match &self {
            Server::Http(s) => Box::new(s.incoming().await?),
            Server::Socks(s) => Box::new(s.incoming().await?),
        })
    }
}

#[derive(Debug)]
pub enum Client {
    Http(http::Client),
    Direct(direct::Client),
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Client::Http(_) => write!(f, "http"),
            Client::Direct(_) => write!(f, "direct"),
        }
    }
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, anyhow::Error> {
        Ok(match config {
            ClientConfig::Http(config) => Self::Http(http::Client::new(config).await?),
            ClientConfig::Direct(config) => Self::Direct(direct::Client::new(config).await),
        })
    }

    #[instrument(skip_all)]
    pub async fn send(&self, req: ProxyRequest) -> Result<ProxyResponse, anyhow::Error> {
        match &self {
            Client::Http(c) => c.send(req).await,
            Client::Direct(c) => c.send(req).await,
        }
    }
}
