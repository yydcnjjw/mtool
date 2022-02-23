use msysev::Event as SysEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::{Service, ServiceRequest, ServiceResponse};

pub struct Sysev {
    tx: broadcast::Sender<SysEvent>,
}

impl Sysev {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel(32);

        let self_ = Arc::new(Self { tx });

        tokio::spawn(Self::run_loop(self_.clone()));

        self_
    }

    async fn run_loop(self: Arc<Self>) {
        let tx = self.tx.clone();
        if let Err(e) = tokio::task::spawn_blocking(move || {
            msysev::run_loop(move |e| {
                if let Err(e) = tx.send(e) {
                    log::warn!("Failed to send sys event: {:?}", e);
                }
            })
        })
        .await
        {
            log::error!("sysev loop exited: {:?}", e);
        }
    }
}

#[mrpc::service]
impl Service for Sysev {
    fn subscribe(self: Arc<Self>) -> broadcast::Receiver<SysEvent> {
        self.tx.subscribe()
    }
}

impl Drop for Sysev {
    fn drop(&mut self) {
        msysev::quit().unwrap();
    }
}
