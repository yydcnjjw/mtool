pub mod direct;
pub mod http;
// mod socks5;

use core::fmt;

use tracing::instrument;

use crate::{
    config::{egress::ClientConfig, ingress::ServerConfig},
    proxy::ProxyRequest,
};

#[derive(Debug)]
pub enum Server {
    Http(http::Server),
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self, anyhow::Error> {
        Ok(match config {
            ServerConfig::Http(config) => Self::Http(http::Server::new(config).await?),
        })
    }

    #[instrument(skip_all)]
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        match &self {
            Server::Http(s) => s.run().await,
        }
    }

    #[instrument(skip_all)]
    pub async fn proxy_accept(&self) -> Result<ProxyRequest, anyhow::Error> {
        match &self {
            Server::Http(s) => s.proxy_accept().await,
        }
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
    pub async fn handle_proxy_request(&self, req: ProxyRequest) -> Result<(), anyhow::Error> {
        match &self {
            Client::Http(c) => c.handle_proxy_request(req).await,
            Client::Direct(c) => c.handle_proxy_request(req).await,
        }
    }
}
