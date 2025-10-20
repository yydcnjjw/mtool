use mapp::{anyhow, prelude::*, tokio::sync::broadcast};

use crate::SystemEvent;

pub struct SystemEventSource {
    source: broadcast::Sender<SystemEvent>,
}

impl SystemEventSource {
    pub async fn new() -> Result<SystemEventSource, anyhow::Error> {
        let (source, _) = broadcast::channel(64);

        Ok(SystemEventSource { source })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.source.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.source.clone()
    }
}
