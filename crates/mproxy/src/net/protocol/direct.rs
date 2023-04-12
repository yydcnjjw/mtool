use tokio::net::TcpStream;

use crate::{
    config::egress::direct::ClientConfig,
    proxy::{ProxyConn, ProxyRequest, ProxyResponse},
};

#[derive(Debug)]
pub struct Client {}

impl Client {
    pub async fn new(_: ClientConfig) -> Self {
        Self {}
    }

    pub async fn send(&self, req: ProxyRequest) -> Result<ProxyResponse, anyhow::Error> {
        let remote = req.remote.to_string();
        let (upload_bytes, download_bytes) = match req.conn {
            ProxyConn::ForwardTcp(conn) => conn.forward(TcpStream::connect(remote).await?).await?,
            ProxyConn::ForwardHttp(conn) => conn.forward(TcpStream::connect(remote).await?).await?,
        };
        Ok(ProxyResponse {
            upload_bytes,
            download_bytes,
        })
    }
}
