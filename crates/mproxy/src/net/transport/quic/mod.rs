mod bistream;

use anyhow::Context;
use quinn::{congestion::{BbrConfig, CubicConfig, NewRenoConfig}, rustls};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug_span, error, info, instrument, warn, Instrument};

use crate::{
    config::transport::quic::{
        AcceptorConfig, CongestionType, ConnectorConfig, StatsConfig, TransportConfig,
    },
    net::tool::dynamic_port,
};

use self::bistream::BiStream;

impl From<TransportConfig> for quinn::TransportConfig {
    fn from(config: TransportConfig) -> Self {
        let mut transport_config = quinn::TransportConfig::default();
        if let Some(t) = config.congestion {
            match t {
                CongestionType::Bbr { initial_window } => {
                    let mut bbr = BbrConfig::default();

                    if let Some(initial_window) = initial_window {
                        bbr.initial_window(initial_window);
                    }

                    info!("congestion params: {:?}", bbr);

                    transport_config.congestion_controller_factory(Arc::new(bbr))
                }
                CongestionType::Cubic => {
                    transport_config.congestion_controller_factory(Arc::new(CubicConfig::default()))
                }
                CongestionType::NewReno => transport_config
                    .congestion_controller_factory(Arc::new(NewRenoConfig::default())),
            };
        }

        transport_config
            .keep_alive_interval(config.keep_alive_interval.map(|v| Duration::from_secs(v)));
        transport_config
    }
}

fn record_stats(stats: Option<StatsConfig>, conn: Arc<quinn::Connection>) {
    if let Some(stats) = &stats {
        let interval = Duration::from_secs(stats.interval as u64);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                if let Some(_) = conn.close_reason() {
                    break;
                }

                let congestion = conn.congestion_state();
                info!(
                    stats = format!("{:?}", conn.stats()),
                    window = congestion.window(),
                    "quic connection"
                );
            }
        });
    }
}

#[derive(Debug)]
pub struct Acceptor {
    _endpoint: quinn::Endpoint,
    sock_rx: Mutex<mpsc::UnboundedReceiver<BiStream>>,
}

impl Acceptor {
    pub async fn new(config: AcceptorConfig) -> Result<Self, anyhow::Error> {
        let tls_config = rustls::ServerConfig::try_from(&config.tls)?;

        let mut quic_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));

        quic_config.transport_config(Arc::new(quinn::TransportConfig::from(config.transport)));

        let endpoint = quinn::Endpoint::server(quic_config, config.listen)?;

        info!("Listening on {}", config.listen);

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(Self::run(tx, endpoint.clone(), config.stats));

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
    pub async fn run(
        tx: mpsc::UnboundedSender<BiStream>,
        endpoint: quinn::Endpoint,
        stats: Option<StatsConfig>,
    ) {
        while let Some(conn) = endpoint.accept().await {
            let tx = tx.clone();
            let stats = stats.clone();
            tokio::spawn(
                async move {
                    let connection = match conn.await {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to establish the connection: {:?}", e);
                            return;
                        }
                    };

                    let conn = Arc::new(connection);

                    record_stats(stats, conn.clone());

                    loop {
                        match BiStream::accept(&conn).await {
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
    inner: dynamic_port::Connector<ConnectorInner, BiStream>,
}

impl Connector {
    pub async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        let endpoint = config.endpoint.clone();
        Ok(Self {
            inner: dynamic_port::Connector::new(ConnectorInner::new(config).await?, endpoint)
                .await?,
        })
    }

    #[instrument(skip_all, fields(transport = "quic"))]
    pub async fn connect(&self) -> Result<BiStream, anyhow::Error> {
        self.inner.connect().await
    }
}

#[derive(Debug)]
pub struct ConnectorInner {
    endpoint: quinn::Endpoint,
    conn: RwLock<Option<Arc<quinn::Connection>>>,
    server_name: String,
    stats: Option<StatsConfig>,
}

impl ConnectorInner {
    async fn new(config: ConnectorConfig) -> Result<Self, anyhow::Error> {
        let tls_config = rustls::ClientConfig::try_from(&config.tls)?;

        let mut quic_config = quinn::ClientConfig::new(Arc::new(tls_config));
        quic_config.transport_config(Arc::new(quinn::TransportConfig::from(config.transport)));

        let mut endpoint = quinn::Endpoint::client(config.local.clone())?;
        endpoint.set_default_client_config(quic_config.clone());

        Ok(Self {
            endpoint,
            conn: RwLock::new(None),
            server_name: config.server_name.to_string(),
            stats: config.stats,
        })
    }
}

impl ConnectorInner {
    async fn open_bistream(&self) -> Result<BiStream, anyhow::Error> {
        if let Some(conn) = self.conn.read().await.as_ref() {
            BiStream::open(conn).await
        } else {
            anyhow::bail!("connection is invalid")
        }
    }
}
