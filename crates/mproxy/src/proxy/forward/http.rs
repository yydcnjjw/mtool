use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task,
};

use anyhow::Context;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{self, Bytes},
    client::conn::http1,
    header, Request, Response, Uri,
};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::oneshot,
};
use tracing::warn;

use crate::stats::{Copyed, GetTransferStats, TransferMonitor};

#[derive(Debug)]
pub struct HttpForwarder {
    pub req: Request<body::Incoming>,
    pub resp_tx: oneshot::Sender<Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error>>,
    pub remove_proxy_header: bool,
}

impl HttpForwarder {
    pub fn new(
        req: Request<body::Incoming>,
        resp_tx: oneshot::Sender<Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error>>,
    ) -> Self {
        Self {
            req,
            resp_tx,
            remove_proxy_header: true,
        }
    }

    pub async fn forward_inner<StreamIO>(
        mut req: Request<body::Incoming>,
        s: StreamIO,
        remove_proxy_header: bool,
    ) -> Result<Response<body::Incoming>, anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (mut sender, conn) = http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(TokioIo::new(s))
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                warn!("Connection failed: {:?}", err);
            }
        });

        if remove_proxy_header {
            *req.uri_mut() = Uri::builder()
                .path_and_query(
                    req.uri()
                        .path_and_query()
                        .context(format!("get path and query from {}", req.uri().to_string()))?
                        .to_string(),
                )
                .build()?;
            Self::remove_proxy_headers(&mut req);
        }

        Ok(sender.send_request(req).await?)
    }

    pub async fn forward<StreamIO>(self, s: StreamIO) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        self.forward_with_monitor_inner(s, None).await
    }

    pub async fn forward_with_monitor<StreamIO>(
        self,
        s: StreamIO,
        monitor: &TransferMonitor,
    ) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        self.forward_with_monitor_inner(s, Some(monitor)).await
    }

    async fn forward_with_monitor_inner<StreamIO>(
        self,
        s: StreamIO,
        monitor: Option<&TransferMonitor>,
    ) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let Self {
            req,
            resp_tx,
            remove_proxy_header,
        } = self;

        let (tx, rx) = oneshot::channel();
        let s = StreamWrapper::new(s, tx);

        let copyed = Arc::new(Copyed::new(s.copyed_ref()));
        if let Some(monitor) = monitor {
            monitor.bind(copyed.clone()).await;
        }

        resp_tx
            .send(
                Self::forward_inner(req, s, remove_proxy_header)
                    .await
                    .map(|resp| resp.map(|b| b.boxed())),
            )
            .map_err(|e| anyhow::anyhow!("{:?} is dropped", e))?;

        let _ = rx.await.context("Waiting for forwarding to be completed")?;

        let stats = copyed.get_transfer_stats();
        Ok((stats.tx as u64, stats.rx as u64))
    }

    fn remove_proxy_headers<Body>(req: &mut Request<Body>) {
        let headers = req.headers_mut();
        headers.remove(header::ACCEPT_ENCODING);
        headers.remove(header::CONNECTION);
        headers.remove("proxy-connection");
        headers.remove(header::PROXY_AUTHENTICATE);
        headers.remove(header::PROXY_AUTHORIZATION);
    }
}

struct StreamWrapper<StreamIO> {
    io: StreamIO,
    upload_bytes: Arc<AtomicU64>,
    download_bytes: Arc<AtomicU64>,
    tx: Option<oneshot::Sender<()>>,
}

impl<StreamIO> StreamWrapper<StreamIO> {
    fn new(io: StreamIO, tx: oneshot::Sender<()>) -> Self {
        Self {
            io,
            upload_bytes: Arc::new(AtomicU64::new(0)),
            download_bytes: Arc::new(AtomicU64::new(0)),
            tx: Some(tx),
        }
    }

    fn copyed_ref(&self) -> (Arc<AtomicU64>, Arc<AtomicU64>) {
        (self.upload_bytes.clone(), self.download_bytes.clone())
    }
}

impl<StreamIO> Drop for StreamWrapper<StreamIO> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }
}

impl<StreamIO> AsyncRead for StreamWrapper<StreamIO>
where
    StreamIO: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> task::Poll<std::io::Result<()>> {
        let result = Pin::new(&mut self.io).poll_read(cx, buf);
        match &result {
            task::Poll::Ready(r) => {
                if r.is_ok() {
                    self.download_bytes
                        .fetch_add(buf.filled().len() as u64, Ordering::Relaxed);
                }
            }
            task::Poll::Pending => {}
        }
        result
    }
}

impl<StreamIO> AsyncWrite for StreamWrapper<StreamIO>
where
    StreamIO: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> task::Poll<Result<usize, std::io::Error>> {
        self.upload_bytes
            .fetch_add(buf.len() as u64, Ordering::Relaxed);
        Pin::new(&mut self.io).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.io).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.io).poll_shutdown(cx)
    }
}
