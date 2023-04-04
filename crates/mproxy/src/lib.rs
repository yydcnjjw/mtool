#![feature(trait_alias)]

mod config;
mod io;
mod net;
mod proxy;
mod router;
mod admin;

// #[cfg(feature = "telemetry")]
pub mod metrics;

use anyhow::Context;
use futures::{future::try_join_all, FutureExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug_span, error, info, warn, Instrument};

pub use config::AppConfig;
use proxy::{Egress, Ingress, ProxyRequest};
use router::Router;

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

                    info!("request {}, {} => {}", req.remote, source, dest);

                    tokio::spawn(
                        async move {
                            if let Err(e) = egress.handle_proxy_request(req).await {
                                warn!("handle proxy request failed: {:?}", e);
                            }
                        }
                        .instrument(debug_span!("handle_proxy_request")),
                    );
                }
                Err(e) => warn!("routing failed: {}", e),
            }
        }
        Ok(())
    }
}
