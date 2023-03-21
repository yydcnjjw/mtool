use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body, client::conn::http1, header, Request, Response};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::oneshot,
};
use tracing::warn;

#[derive(Debug)]
pub struct ForwardHttpConn {
    pub req: Request<body::Incoming>,
    pub resp_tx: oneshot::Sender<Response<BoxBody<Bytes, hyper::Error>>>,
    pub remove_proxy_header: bool,
}

impl ForwardHttpConn {
    pub fn new(
        req: Request<body::Incoming>,
        resp_tx: oneshot::Sender<Response<BoxBody<Bytes, hyper::Error>>>,
    ) -> Self {
        Self {
            req,
            resp_tx,
            remove_proxy_header: true,
        }
    }

    pub async fn forward<StreamIO>(mut self, s: StreamIO) -> Result<(), anyhow::Error>
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

        if self.remove_proxy_header {
            Self::remove_proxy_headers(&mut self.req);
        }

        let resp = sender.send_request(self.req).await?;
        self.resp_tx.send(resp.map(|b| b.boxed())).unwrap();
        Ok(())
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
