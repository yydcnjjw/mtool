use std::env::consts::OS;

use mapp::{
    anyhow,
    futures::TryFutureExt,
    prelude::*,
    tokio::{self, sync::broadcast},
    tracing::warn,
};
use mtool_core::ConfigStore;

use crate::{
    config::Config,
    p2p::{RemoteSystemEvent, RemoteSystemEventSource},
    platform, SystemEvent,
};

pub struct SystemEventSource {
    config: Config,
    pub(crate) inner: platform::SystemEventSource,
}

impl SystemEventSource {
    pub async fn construct(
        remote_source: Res<RemoteSystemEventSource>,
        injector: Injector,
        cs: Res<ConfigStore>,
    ) -> Result<Res<SystemEventSource>, anyhow::Error> {
        let source = SystemEventSource {
            inner: inject_once(&injector, platform::SystemEventSource::new).await??,
            config: cs.get_optional("system").unwrap_or_default(),
        };

        tokio::spawn(Self::broadcast_to_remote(source.subscribe(), remote_source));

        Ok(Res::new(source))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.inner.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.inner.sender()
    }

    async fn broadcast_to_remote(
        mut rx: broadcast::Receiver<SystemEvent>,
        remote_source: Res<RemoteSystemEventSource>,
    ) {
        while let Ok(event) = rx.recv().await {
            match event {
                // TODO: filter based on config
                SystemEvent::NotificationPosted(_) => {
                    let remote_source = remote_source.clone();
                    tokio::spawn(async move {
                        remote_source
                            .publish(&RemoteSystemEvent {
                                source: OS.to_owned(),
                                event,
                            })
                            .inspect_err(|e| warn!("{e:?}"))
                            .await
                    });
                }
                _ => {}
            }
        }
    }
}

// trait SystemEventSource {
//     fn subscribe(&self) -> broadcast::Receiver<SystemEvent>;

//     fn publish(&self) -> broadcast::Sender<SystemEvent>;

//     fn stream(&self) -> BroadcastStream<SystemEvent> {
//         BroadcastStream::new(self.subscribe())
//     }
// }
