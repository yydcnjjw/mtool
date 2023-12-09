use std::{pin::Pin, task, io, sync::Arc, net::SocketAddr};

use quinn::{RecvStream, SendStream, VarInt};
use tokio::io::{AsyncWrite, AsyncRead};

use crate::net::transport::Connect;

use super::{ConnectorInner, record_stats};

#[derive(Debug)]
pub struct BiStream {
    r: RecvStream,
    w: SendStream,
}

impl BiStream {
    pub async fn accept(conn: &quinn::Connection) -> Result<Self, anyhow::Error> {
        let (w, r) = conn.accept_bi().await?;
        Ok(Self { r, w })
    }

    pub async fn open(conn: &quinn::Connection) -> Result<Self, anyhow::Error> {
        let (w, r) = conn.open_bi().await?;
        Ok(Self { r, w })
    }
}

impl AsyncWrite for BiStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> task::Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.w).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), io::Error>> {
        Pin::new(&mut self.w).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), io::Error>> {
        Pin::new(&mut self.w).poll_shutdown(cx)
    }
}

impl AsyncRead for BiStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        Pin::new(&mut self.r).poll_read(cx, buf)
    }
}

impl Connect<BiStream> for ConnectorInner {
    async fn is_open(&self) -> bool {
        let conn = self.conn.read().await;
        if let Some(conn) = conn.as_ref() {
            conn.close_reason().is_none()
        } else {
            false
        }
    }

    async fn connect(&self, endpoint: SocketAddr) -> Result<(), anyhow::Error> {
        let conn = Arc::new(self.endpoint.connect(endpoint, &self.server_name)?.await?);

        record_stats(self.stats.clone(), conn.clone());

        *self.conn.write().await = Some(conn);

        Ok(())
    }

    async fn open_stream(&self) -> Result<BiStream, anyhow::Error> {
        self.open_bistream().await
    }

    async fn close(&self) {
        if !self.is_open().await {
            return;
        }
        if let Some(conn) = self.conn.write().await.as_mut() {
            conn.close(VarInt::from_u32(0), &[]);
        }
    }
}
