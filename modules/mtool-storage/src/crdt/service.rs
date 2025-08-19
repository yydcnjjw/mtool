use loro::{ExportMode, LoroDoc, Subscription, VersionVector};
use mapp::{
    anyhow,
    futures::{Stream, StreamExt},
    prelude::*,
    tokio::{self, sync::broadcast},
    tokio_stream::wrappers::BroadcastStream,
    tracing::{debug, warn},
};
use mtool_core::ConfigStore;
use std::{pin::Pin, sync::Arc};

use crate::rpc::{
    self,
    pb::{self, CrdtDocVersion, CrdtSyncMessage, CrdtUpdates},
};

#[derive(Clone)]
pub struct CrdtService {
    pub(crate) store: LoroDoc,

    channel: broadcast::Sender<CrdtSyncMessage>,
    _subscription: Arc<Subscription>,
}

pub type CrdtStream = Pin<Box<dyn Stream<Item = Result<CrdtSyncMessage, tonic::Status>> + Send>>;

impl CrdtService {
    async fn sync_remote(this: Res<Self>, cs: Res<ConfigStore>) {
        if let Err(e) = Self::sync_remote_inner(this, cs).await {
            warn!("{:?}", e);
        }
    }

    async fn sync_remote_inner(this: Res<Self>, cs: Res<ConfigStore>) -> Result<(), anyhow::Error> {
        let server = cs.get::<String>("storage.crdt.remote").await?;

        let mut rpc = rpc::MtoolStorageClient::connect(server).await?;

        let store = this.store.clone();

        let peer_updates = &rpc
            .crdt_pull_peer_updates(CrdtDocVersion {
                data: store.oplog_vv().encode(),
            })
            .await?
            .into_inner();

        store.import(&peer_updates.updates)?;

        debug!("{:?}", store.analyze());

        let peer_version = VersionVector::decode(&peer_updates.version)?;
        rpc.crdt_push_local_updates(CrdtUpdates {
            version: store.oplog_vv().encode(),
            updates: store.export(ExportMode::updates(&peer_version))?,
        })
        .await?;

        let id = store.peer_id();
        let req = tonic::Request::new(BroadcastStream::new(this.channel.subscribe()).filter_map(
            move |msg| {
                let id = id.clone();
                async move {
                    match msg {
                        Ok(msg) => (msg.id == id).then(|| msg),
                        Err(e) => {
                            warn!("{:?}", e);
                            None
                        }
                    }
                }
            },
        ));

        let mut rx = rpc.crdt_sync(req).await?.into_inner();
        tokio::spawn(async move {
            while let Some(updates) = rx.next().await {
                match updates {
                    Ok(CrdtSyncMessage { updates, .. }) => {
                        if let Err(e) = store.import(&updates) {
                            warn!("{:?}", e);
                            break;
                        }

                        debug!("{:?}", store.analyze());
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let (channel, _) = broadcast::channel(64);
        let store = LoroDoc::new();

        let subscription = {
            let sender = channel.clone();
            let id = store.peer_id();
            store.subscribe_local_update(Box::new(move |e| {
                _ = sender.send(CrdtSyncMessage {
                    id,
                    updates: e.to_vec(),
                });
                true
            }))
        };

        let this = Res::new(Self {
            channel,
            store,
            _subscription: Arc::new(subscription),
        });

        tokio::spawn(Self::sync_remote(this.clone(), cs));

        Ok(this)
    }

    pub async fn handle_pull_peer_updates(
        &self,
        peer_version: CrdtDocVersion,
    ) -> Result<CrdtUpdates, anyhow::Error> {
        Ok(CrdtUpdates {
            version: self.store.oplog_vv().encode(),
            updates: self
                .store
                .export(ExportMode::updates(&VersionVector::decode(
                    &peer_version.data,
                )?))?,
        })
    }

    pub async fn handle_push_local_updates(
        &self,
        updates: CrdtUpdates,
    ) -> Result<pb::Empty, anyhow::Error> {
        self.store.import(&updates.updates)?;
        debug!("{:?}", self.store.analyze());
        Ok(pb::Empty {})
    }

    pub async fn handle_sync(
        &self,
        mut stream: tonic::Streaming<CrdtSyncMessage>,
    ) -> Result<CrdtStream, anyhow::Error> {
        let store = self.store.clone();
        let sender = self.channel.clone();

        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) = store.import(&msg.updates) {
                            warn!("{:?}", e);
                            break;
                        }

                        debug!("{:?}", store.analyze());

                        _ = sender.send(msg);
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                        break;
                    }
                }
            }
        });

        let stream = {
            let id = self.store.peer_id();
            Box::pin(BroadcastStream::new(self.channel.subscribe()).filter_map(
                move |msg| async move {
                    match msg {
                        Ok(msg) => {
                            if msg.id == id {
                                None
                            } else {
                                Some(Ok(msg))
                            }
                        }
                        Err(e) => Some(Err(tonic::Status::internal(format!("{:?}", e)))),
                    }
                },
            )) as CrdtStream
        };

        Ok(stream)
    }
}
