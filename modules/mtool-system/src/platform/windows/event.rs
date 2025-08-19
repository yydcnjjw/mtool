#[allow(unused_imports)]
use mapp::{anyhow, itertools::Itertools, prelude::*, tokio::sync::broadcast, tracing::warn};
use mtool_dioxus::prelude::*;
// use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

use crate::SystenEvent;

pub struct SystemEventSource {
    source: broadcast::Sender<SystenEvent>,
}

impl SystemEventSource {
    pub async fn new(_context: Res<DioxusContext>) -> Result<SystemEventSource, anyhow::Error> {
        // let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;
        // let sessions = manager.GetSessions()?.into_iter().collect_vec();
        // warn!("GetSessions: {:?}", sessions.len());
        // for session in sessions {
        //     let media_properties = session.TryGetMediaPropertiesAsync()?.await?;
        //     warn!("{:?}", media_properties.Title()?);
        //     warn!("{:?}", media_properties.Artist()?);
        // }

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
