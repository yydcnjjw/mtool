use tokio::net::TcpStream;

use crate::{
    config::egress::direct::ClientConfig,
    proxy::{ProxyConn, ProxyRequest, ProxyResponse},
    stats::{TransferMonitor, TransferStats},
};

#[derive(Debug)]
pub struct Client {
    monitor: TransferMonitor,
}

impl Client {
    pub async fn new(_: ClientConfig) -> Self {
        Self {
            monitor: TransferMonitor::new(),
        }
    }

    pub async fn send(&self, req: ProxyRequest) -> Result<ProxyResponse, anyhow::Error> {
        let remote = req.remote.to_string();
        let s = TcpStream::connect(remote).await?;
        let (upload_bytes, download_bytes) = match req.conn {
            ProxyConn::ForwardTcp(conn) => conn.forward_with_monitor(s, &self.monitor).await?,
            ProxyConn::ForwardHttp(conn) => conn.forward_with_monitor(s, &self.monitor).await?,
        };
        Ok(ProxyResponse {
            upload_bytes,
            download_bytes,
        })
    }

    pub async fn get_transfer_stats(&self) -> Result<TransferStats, anyhow::Error> {
        self.monitor.get_transfer_stats().await
    }
}
