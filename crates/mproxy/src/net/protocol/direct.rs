use tokio::net::TcpStream;

use crate::{
    config::egress::direct::ClientConfig,
    proxy::{ProxyConn, ProxyRequest},
};

#[derive(Debug)]
pub struct Client {}

impl Client {
    pub async fn new(_: ClientConfig) -> Self {
        Self {}
    }

    pub async fn handle_proxy_request(&self, req: ProxyRequest) -> Result<(), anyhow::Error> {
        let remote = req.remote.to_string();
        match req.conn {
            ProxyConn::ForwardTcp(conn) => conn.forward(TcpStream::connect(remote).await?).await,
            ProxyConn::ForwardHttp(conn) => conn.forward(TcpStream::connect(remote).await?).await,
        }
    }
}
