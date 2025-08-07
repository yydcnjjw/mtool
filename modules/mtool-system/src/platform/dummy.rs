use mapp::{
    anyhow,
    prelude::*,
    tokio::sync::broadcast,
};
use mtool_dioxus::prelude::*;

use crate::SystenEvent;

pub struct SystemEventSource {
    source: broadcast::Sender<SystenEvent>,
}

impl SystemEventSource {
    pub async fn new(context: Res<DioxusContext>) -> Result<SystemEventSource, anyhow::Error> {
        let (source, _) = broadcast::channel(64);

        Ok(SystemEventSource { source })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystenEvent> {
        self.source.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystenEvent> {
        self.source.clone()
    }
}
