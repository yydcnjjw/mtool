use loro::{ExportMode, LoroDoc, Subscription, VersionVector};
use mapp::{
    anyhow,
    futures::TryFutureExt,
    prelude::*,
    serde::{Deserialize, Serialize},
    tokio,
    tokio_stream::StreamExt,
    tracing::{debug, warn},
};
use mtool_p2p::{gossipsub::IdentTopic, Peer, SubjectMessage};
use std::{
    fmt,
    sync::{Arc, LazyLock},
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
enum Message {
    SyncRequest { version: Vec<u8> },
    SyncReply { version: Vec<u8>, data: Vec<u8> },
    Update { update: Vec<u8> },
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SyncRequest { .. } => f.debug_struct("SyncRequest").finish(),
            Self::SyncReply { .. } => f.debug_struct("SyncReply").finish(),
            Self::Update { .. } => f.debug_struct("Update").finish(),
        }
    }
}

#[derive(Clone)]
pub struct CrdtStore {
    pub(crate) store: LoroDoc,

    _subscription: Arc<Subscription>,
}

static CRDT_TOPIC: LazyLock<IdentTopic> = LazyLock::new(|| IdentTopic::new("CRDT"));

impl CrdtStore {
    pub async fn sync_request(&self, peer: &Res<Peer>) -> Result<(), anyhow::Error> {
        peer.publish(
            &CRDT_TOPIC,
            &Message::SyncRequest {
                version: self.store.oplog_vv().encode(),
            },
        )
        .await
    }

    async fn start_sync(this: Res<Self>, peer: Res<Peer>) -> Result<(), anyhow::Error> {
        let store = this.store.clone();

        let crdt_subject = peer.subscribe::<Message>(&CRDT_TOPIC).await?;
        let crdt_source_subject = peer
            .subscribe::<Message>(&IdentTopic::new(format!("CRDT-SOURCE-{}", peer.id())))
            .await?;

        this.sync_request(&peer).await?;

        let mut stream = crdt_subject.stream().merge(crdt_source_subject.stream());

        while let Some(item) = stream.next().await {
            match item {
                Ok(SubjectMessage {
                    source,
                    topic: _,
                    data,
                }) => {
                    debug!(?data);

                    match data {
                        Message::SyncRequest { version } => {
                            if let Some(source) = source {
                                peer.publish(
                                    &IdentTopic::new(format!("CRDT-SOURCE-{source}")),
                                    &Message::SyncReply {
                                        version: store.oplog_vv().encode(),
                                        data: store.export(ExportMode::updates(
                                            &VersionVector::decode(&version)?,
                                        ))?,
                                    },
                                )
                                .await?;
                            }
                        }
                        Message::SyncReply { version, data } => {
                            _ = store.import(&data)?;
                            if let Some(source) = source {
                                peer.publish(
                                    &IdentTopic::new(format!("CRDT-SOURCE-{source}")),
                                    &Message::Update {
                                        update: store.export(ExportMode::updates(
                                            &VersionVector::decode(&version)?,
                                        ))?,
                                    },
                                )
                                .await?;
                            }
                        }
                        Message::Update { update } => {
                            let status = store.import(&update)?;
                            debug!(?status);
                            if status.pending.is_some() {
                                this.sync_request(&peer).await?;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("{e:?}");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn construct(peer: Res<Peer>) -> Result<Res<Self>, anyhow::Error> {
        let store = LoroDoc::new();

        let rt = tokio::runtime::Handle::current();

        let subscription = {
            let peer = peer.clone();
            store.subscribe_local_update(Box::new(move |e| {
                let peer = peer.clone();
                let update = Message::Update { update: e.to_vec() };
                let _guard = rt.enter();
                tokio::spawn(async move {
                    if let Err(e) = peer.publish(&CRDT_TOPIC, &update).await {
                        warn!("{e:?}")
                    }
                });

                true
            }))
        };

        let this = Res::new(Self {
            store,
            _subscription: Arc::new(subscription),
        });

        tokio::spawn(Self::start_sync(this.clone(), peer).inspect_err(|e| warn!("{e:?}")));

        Ok(this)
    }
}
