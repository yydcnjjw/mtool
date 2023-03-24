use anyhow::Context;
use quinn::{Connection, RecvStream, SendStream};
use std::{io, net::SocketAddr, pin::Pin, sync::Arc, task, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc, Mutex, RwLock},
};
use tracing::{debug_span, error, info, instrument, warn, Instrument};

use crate::config::transport::quic::{AcceptorConfig, ConnectorConfig};

#[derive(Debug)]
pub struct Acceptor {
    _endpoint: quinn::Endpoint,
    sock_rx: Mutex<mpsc::UnboundedReceiver<BiStream>>,
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Self, anyhow::Error> {
        let tls_config = rustls::ServerConfig::try_from(&config.tls)?;

        let quic_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));
        let endpoint = quinn::Endpoint::server(quic_config, config.listen)?;

        info!("Listening on {}", config.listen);

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(Self::run(tx, endpoint.clone()));

        Ok(Self {
            _endpoint: endpoint,
            sock_rx: Mutex::new(rx),
        })
    }

    pub async fn accept(&self) -> Result<BiStream, anyhow::Error> {
        let mut rx = self.sock_rx.lock().await;
        rx.recv().await.context("Failed to accpet")
    }

    #[instrument(skip_all)]
    pub async fn run(tx: mpsc::UnboundedSender<BiStream>, endpoint: quinn::Endpoint) {
        while let Some(conn) = endpoint.accept().await {
            let tx = tx.clone();
            tokio::spawn(
                async move {
                    let connection = match conn.await {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to establish the connection: {:?}", e);
                            return;
                        }
                    };

                    loop {
                        match BiStream::accept(&connection).await {
                            Ok(s) => {
                                if let Err(e) = tx.send(s) {
                                    warn!("{:?}", e);
                                }
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                return;
                            }
                        }
                    }
                }
                .instrument(debug_span!("connection")),
            );
        }
    }
}

#[derive(Debug)]
pub struct Connector {
    endpoint: quinn::Endpoint,
    connection: RwLock<quinn::Connection>,

    remote: SocketAddr,
    server_name: String,
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        let tls_config = rustls::ClientConfig::try_from(&config.tls)?;

        let mut quic_config = quinn::ClientConfig::new(Arc::new(tls_config));

        let mut transport_config = quinn::TransportConfig::default();
        transport_config
            .keep_alive_interval(config.keep_alive_interval.map(|v| Duration::from_secs(v)));

        quic_config.transport_config(Arc::new(transport_config));

        let mut endpoint = quinn::Endpoint::client(config.local.clone())?;
        endpoint.set_default_client_config(quic_config);

        let remote = config.endpoint.clone();
        let connection = endpoint
            .connect(remote.clone(), &config.server_name)?
            .await?;
        Ok(Self {
            endpoint,
            connection: RwLock::new(connection),

            remote,
            server_name: config.server_name.to_string(),
        })
    }
}

impl Connector {
    pub async fn reconnect_quic(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.connection.write().await;
        *conn = self
            .endpoint
            .connect(self.remote.clone(), &self.server_name)?
            .await?;
        Ok(())
    }

    pub async fn open_bistream(&self) -> Result<BiStream, anyhow::Error> {
        let conn = self.connection.read().await;
        BiStream::open(&conn).await
    }

    pub async fn connect(&self) -> Result<BiStream, anyhow::Error> {
        match self.open_bistream().await {
            Ok(s) => Ok(s),
            Err(e) => {
                warn!("quic is disconnect: {}, reconnect", e);
                self.reconnect_quic().await?;
                self.open_bistream().await
            }
        }
    }
}

#[derive(Debug)]
pub struct BiStream {
    r: RecvStream,
    w: SendStream,
}

impl BiStream {
    pub async fn accept(conn: &Connection) -> Result<Self, anyhow::Error> {
        let (w, r) = conn.accept_bi().await?;
        Ok(Self { r, w })
    }

    pub async fn open(conn: &Connection) -> Result<Self, anyhow::Error> {
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
