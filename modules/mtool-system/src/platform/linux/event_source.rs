use mapp::{
    anyhow,
    futures::{StreamExt, TryFutureExt},
    tokio::{self, sync::broadcast},
    tracing::warn,
};
use std::sync::Arc;

use crate::SystemEvent;

use super::x11;

pub struct SystemEventSource {
    source: broadcast::Sender<SystemEvent>,
    _session: Arc<x11::RecordSession>,
}

impl SystemEventSource {
    pub async fn new() -> Result<SystemEventSource, anyhow::Error> {
        let (source, _) = broadcast::channel(64);

        let session = Arc::new(x11::RecordSession::new().await?);

        {
            let session = session.clone();
            let source = source.clone();
            tokio::spawn(
                async move {
                    let mut stream = session.event_stream().await?;
                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(event) => {
                                _ = source.send(event);
                            }
                            Err(e) => {
                                warn!("{e:?}");
                                break;
                            }
                        }
                    }
                    Ok::<_, anyhow::Error>(())
                }
                .unwrap_or_else(|e| warn!("{e:?}")),
            );
        }

        Ok(SystemEventSource {
            source,
            _session: session,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.source.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.source.clone()
    }
}
