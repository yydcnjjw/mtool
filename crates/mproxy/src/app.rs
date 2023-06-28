use anyhow::Context;
use futures::{future::try_join_all, FutureExt};
use std::{sync::Arc, time::Instant};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tracing::{info, info_span, warn, Instrument};

use super::{
    proxy::{Egress, Ingress, ProxyRequest, ProxyResponse},
    router::Router,
    stats::Stats,
    AppConfig
};

#[derive(Debug)]
pub struct App {
    ingress: Vec<Arc<Ingress>>,
    egress: Vec<Arc<Egress>>,
    router: Router,
    tx: mpsc::UnboundedSender<(String, ProxyRequest)>,
    rx: Mutex<mpsc::UnboundedReceiver<(String, ProxyRequest)>>,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            ingress: try_join_all(
                config
                    .ingress
                    .into_iter()
                    .map(|config| Ingress::new(config).map(|v| v.map(|v| Arc::new(v)))),
            )
            .await?,
            egress: try_join_all(
                config
                    .egress
                    .into_iter()
                    .map(|config| Egress::new(config).map(|v| v.map(|v| Arc::new(v)))),
            )
            .await?,
            router: Router::new(config.routing)?,
            tx,
            rx: Mutex::new(rx),
        })
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        self.run_ingress();
        self.dispatch().await
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub async fn stats(&self) -> Result<Stats, anyhow::Error> {
        let mut stats = Stats::default();
        for egress in self.egress.iter() {
            stats
                .transfer
                .insert(egress.id.clone(), egress.get_transfor_stats().await?);
        }
        Ok(stats)
    }

    fn run_ingress(&self) {
        for ingress in &self.ingress {
            let tx = self.tx.clone();
            let ingress = ingress.clone();
            tokio::spawn(async move {
                match ingress.incoming().await {
                    Ok(mut incoming) => {
                        while let Some(request) = incoming.next().await {
                            if let Err(e) = tx.send((ingress.id.clone(), request)) {
                                warn!("{:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                }
            });
        }
    }

    async fn dispatch(&self) -> Result<(), anyhow::Error> {
        while let Some((source, req)) = self.rx.lock().await.recv().await {
            match self.router.route(&source, &req.remote.address) {
                Ok(dest) => {
                    let egress = self
                        .egress
                        .iter()
                        .find(|egress| egress.id == dest)
                        .context(format!("Egress {} isn't exist", dest))?
                        .clone();

                    let remote = req.remote.clone();

                    let span = {
                        let dest = dest.clone();
                        info_span!(
                            "handle_proxy_request",
                            remote = remote.to_string(),
                            source,
                            dest,
                        )
                    };
                    tokio::spawn(
                        async move {
                            info!("start processing proxy request");
                            let now = Instant::now();
                            match egress.send(req).await {
                                Ok(ProxyResponse {
                                    upload_bytes,
                                    download_bytes,
                                }) => {
                                    info!(
                                        spent_time = format!("{}ms", now.elapsed().as_millis()),
                                        upload_bytes, download_bytes, "proxy request finished"
                                    );
                                }
                                Err(e) => {
                                    warn!("proxy request error: {:?}", e);
                                }
                            }
                        }
                        .instrument(span),
                    );
                }
                Err(e) => warn!("routing failed: {}", e),
            }
        }
        Ok(())
    }
}
