use std::env::consts::OS;

use mapp::{
    anyhow,
    futures::TryFutureExt,
    prelude::*,
    tokio::{self, sync::broadcast},
    tracing::warn,
};

use crate::{
    p2p::{RemoteSystemEvent, RemoteSystemEventSource},
    platform, SystemEvent,
};

// trait SystemEventSource {
//     fn subscribe(&self) -> broadcast::Receiver<SystemEvent>;

//     fn publish(&self) -> broadcast::Sender<SystemEvent>;

//     fn stream(&self) -> BroadcastStream<SystemEvent> {
//         BroadcastStream::new(self.subscribe())
//     }
// }

pub struct SystemEventSource {
    pub(crate) inner: platform::SystemEventSource,
}

impl SystemEventSource {
    pub async fn construct(
        remote_source: Res<RemoteSystemEventSource>,
        injector: Injector,
    ) -> Result<Res<SystemEventSource>, anyhow::Error> {
        let source = SystemEventSource {
            inner: inject_once(&injector, platform::SystemEventSource::new).await??,
        };

        tokio::spawn(Self::broadcast_remote(source.subscribe(), remote_source));

        Ok(Res::new(source))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.inner.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.inner.sender()
    }

    async fn broadcast_remote(
        mut rx: broadcast::Receiver<SystemEvent>,
        remote_source: Res<RemoteSystemEventSource>,
    ) {
        while let Ok(event) = rx.recv().await {
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
    }
}
