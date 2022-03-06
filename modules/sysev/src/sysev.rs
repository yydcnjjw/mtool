use msysev::Event as SysEvent;
use std::{sync::Arc, thread};
use tokio::sync::broadcast;

use crate::{Service, ServiceRequest, ServiceResponse};

pub struct Sysev {
    tx: broadcast::Sender<SysEvent>,
}

impl Sysev {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel(32);

        let self_ = Arc::new(Self { tx });

        Self::run_loop(self_.clone());

        self_
    }

    fn run_loop(self: Arc<Self>) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            if let Err(e) = msysev::run_loop(move |e| {
                if let Err(e) = tx.send(e) {
                    log::warn!("Failed to send sys event: {:?}", e);
                }
            }) {
                log::error!("sysev loop exited: {:?}", e);
            }
        });
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
        if let Err(e) = msysev::quit() {
            log::error!("Failed to exit sysev loop: {:?}", e);
        }
    }
}
