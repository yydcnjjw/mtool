use std::{pin::Pin, task, str::FromStr};

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{self, Bytes},
    client::conn::http1,
    header, Request, Response, Uri,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::oneshot,
};
use tracing::warn;

#[derive(Debug)]
pub struct ForwardHttpConn {
    pub req: Request<body::Incoming>,
    pub resp_tx: oneshot::Sender<Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error>>,
    pub remove_proxy_header: bool,
}

impl ForwardHttpConn {
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
            .handshake(s)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                warn!("Connection failed: {:?}", err);
            }
        });

        if remove_proxy_header {
            *req.uri_mut() = Uri::from_str(req.uri().path())?;
            Self::remove_proxy_headers(&mut req);
        }

        Ok(sender.send_request(req).await?)
    }

    pub async fn forward<StreamIO>(self, s: StreamIO) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let Self {
            req,
            resp_tx,
            remove_proxy_header,
        } = self;

        let (tx, rx) = oneshot::channel();

        resp_tx
            .send(
                Self::forward_inner(req, StreamWrapper::new(s, tx), remove_proxy_header)
                    .await
                    .map(|resp| resp.map(|b| b.boxed())),
            )
            .map_err(|e| anyhow::anyhow!("{:?} is dropped", e))?;

        Ok(rx
            .await
            .map(|(upload_bytes, download_bytes)| (upload_bytes as u64, download_bytes as u64))
            .unwrap_or_default())
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
    upload_bytes: usize,
    download_bytes: usize,
    tx: Option<oneshot::Sender<(usize, usize)>>,
}

impl<StreamIO> StreamWrapper<StreamIO> {
    fn new(io: StreamIO, tx: oneshot::Sender<(usize, usize)>) -> Self {
        Self {
            io,
            upload_bytes: 0,
            download_bytes: 0,
            tx: Some(tx),
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
                    self.download_bytes += buf.filled().len();
                }
            }
            task::Poll::Pending => {}
        }
        result
    }
}

impl<StreamIO> Drop for StreamWrapper<StreamIO> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send((self.upload_bytes, self.download_bytes));
        }
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
        self.upload_bytes += buf.len();
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
