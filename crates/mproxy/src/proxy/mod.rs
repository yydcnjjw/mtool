mod forward;
mod net_location;

use std::ops::Deref;

pub use forward::*;
pub use net_location::*;

use crate::{
    config::{egress::EgressConfig, ingress::IngressConfig},
    net::protocol,
};

#[derive(Debug)]
pub struct ProxyRequest {
    pub remote: NetLocation,
    pub conn: ProxyConn,
}

#[derive(Debug)]
pub enum ProxyConn {
    ForwardTcp(ForwardTcpConn),
    ForwardHttp(ForwardHttpConn),
}

#[derive(Debug)]
pub struct Ingress {
    pub id: String,
    server: protocol::Server,
}

impl Ingress {
    pub async fn new(config: IngressConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            id: config.id,
            server: protocol::Server::new(config.server).await?,
        })
    }
}

impl Deref for Ingress {
    type Target = protocol::Server;

    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

#[derive(Debug)]
pub struct Egress {
    pub id: String,

    client: protocol::Client,
}

impl Egress {
    pub async fn new(config: EgressConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            id: config.id,
            client: protocol::Client::new(config.client).await?,
        })
    }
}

impl Deref for Egress {
    type Target = protocol::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
