#[allow(unused_imports)]
use mapp::{anyhow, itertools::Itertools, prelude::*, tokio::sync::broadcast, tracing::warn};
use mtool_dioxus::prelude::*;

use crate::SystemEvent;

pub struct SystemEventSource {
    source: broadcast::Sender<SystemEvent>,
}

impl SystemEventSource {
    pub async fn new(_context: Res<DioxusContext>) -> Result<SystemEventSource, anyhow::Error> {
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
