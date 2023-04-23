#![feature(trait_alias)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

mod admin;
mod config;
mod io;
mod net;
mod proxy;
pub mod router;

#[cfg(feature = "telemetry")]
pub mod metrics;

use anyhow::Context;
use futures::{future::try_join_all, FutureExt};
use std::{sync::Arc, time::Instant};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, info_span, warn, Instrument};

pub use config::AppConfig;
use proxy::{Egress, Ingress, ProxyRequest};
use router::Router;

use crate::proxy::ProxyResponse;

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

    fn run_ingress(&self) {
        for ingress in &self.ingress {
            {
                let ingress = ingress.clone();
                tokio::spawn(async move {
                    if let Err(e) = ingress.run().await {
                        error!("{:?}", e);
                    }
                });
            }

            {
                let tx = self.tx.clone();

                let ingress = ingress.clone();
                tokio::spawn(async move {
                    loop {
                        match ingress.proxy_accept().await {
                            Ok(req) => {
                                if let Err(e) = tx.send((ingress.id.clone(), req)) {
                                    warn!("{:?}", e);
                                }
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                break;
                            }
                        }
                    }
                });
            }
        }
    }

    async fn dispatch(&self) -> Result<(), anyhow::Error> {
        while let Some((source, req)) = self.rx.lock().await.recv().await {
            match self.router.route(&source, &req.remote) {
                Ok(dest) => {
                    let egress = self
                        .egress
                        .iter()
                        .find(|egress| egress.id == dest)
                        .context(format!("Egress {} isn't exist", dest))?
                        .clone();

                    let remote = req.remote.clone();
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
                        .instrument(info_span!(
                            "handle_proxy_request",
                            remote = remote.to_string(),
                            source,
                            dest,
                        )),
                    );
                }
                Err(e) => warn!("routing failed: {}", e),
            }
        }
        Ok(())
    }
}
