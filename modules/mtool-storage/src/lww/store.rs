use loro::{
    awareness::{EphemeralEventTrigger, EphemeralStore, EphemeralStoreEvent},
    LoroValue, Subscription,
};
use mapp::{
    anyhow,
    dashmap::DashMap,
    futures::TryFutureExt,
    itertools::Itertools,
    prelude::*,
    serde::{Deserialize, Serialize},
    tokio,
    tokio_stream::StreamExt,
    tracing::{debug, warn},
};
use mtool_p2p::{gossipsub::IdentTopic, Peer, SubjectMessage};
use std::{
    ops::Deref,
    sync::{Arc, LazyLock},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
enum Message {
    SyncRequest,
    Update { updates: Vec<(String, Vec<u8>)> },
}

pub(crate) type ValueChangedSubscriber = Box<dyn Fn(LoroValue) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct LwwStore {
    store: EphemeralStore,

    value_changed_subscribers: Arc<DashMap<String, ValueChangedSubscriber>>,

    _subscription: Arc<Subscription>,
}

static LWW_TOPIC: LazyLock<IdentTopic> = LazyLock::new(|| IdentTopic::new("LWW"));

impl LwwStore {
    pub async fn construct(peer: Res<Peer>) -> Result<Res<Self>, anyhow::Error> {
        let store = EphemeralStore::new(i64::max_value());
        let value_changed_subscribers = Arc::new(DashMap::<String, ValueChangedSubscriber>::new());

        let rt = tokio::runtime::Handle::current();

        let subscription = {
            let peer = peer.clone();
            let s = store.clone();
            let value_changed_subscribers = value_changed_subscribers.clone();
            store.subscribe(Box::new(
                move |EphemeralStoreEvent {
                          by, added, updated, ..
                      }| {
                    warn!(?by, ?added, ?updated);
                    match by {
                        EphemeralEventTrigger::Local => {
                            let peer = peer.clone();

                            let update = Message::Update {
                                updates: updated
                                    .iter()
                                    .chain(added.iter())
                                    .map(|key| (key.clone(), s.encode(key)))
                                    .collect(),
                            };
                            let _guard = rt.enter();
                            tokio::spawn(async move {
                                if let Err(e) = peer.publish(&LWW_TOPIC, &update).await {
                                    warn!("{e:?}")
                                }
                            });
                        }
                        EphemeralEventTrigger::Import => {
                            warn!(
                                "{:?}",
                                value_changed_subscribers
                                    .iter()
                                    .map(|kv| kv.key().clone())
                                    .collect_vec()
                            );
                            for key in updated.iter().chain(added.iter()) {
                                if let Some(cb) = value_changed_subscribers.get(key) {
                                    if let Some(value) = s.get(key) {
                                        if (cb)(value) {
                                            value_changed_subscribers.remove(key);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    true
                },
            ))
        };

        let this = Res::new(Self {
            store,
            value_changed_subscribers,
            _subscription: Arc::new(subscription),
        });

        tokio::spawn(Self::start_sync(this.clone(), peer).inspect_err(|e| warn!("{e:?}")));

        Ok(this)
    }

    pub(crate) fn subscribe_value_changed<Callback>(&self, key: String, callback: Callback)
    where
        Callback: Fn(LoroValue) -> bool + Send + Sync + 'static,
    {
        self.value_changed_subscribers
            .insert(key, Box::new(callback));
    }

    async fn start_sync(this: Res<Self>, peer: Res<Peer>) -> Result<(), anyhow::Error> {
        let store = this.store.clone();

        let lww_subject = peer.subscribe::<Message>(&LWW_TOPIC).await?;
        let lww_source_subject = peer
            .subscribe::<Message>(&IdentTopic::new(format!("LWW-SOURCE-{}", peer.id())))
            .await?;

        peer.publish(&LWW_TOPIC, &Message::SyncRequest).await?;

        let mut stream = lww_subject.stream().merge(lww_source_subject.stream());

        while let Some(item) = stream.next().await {
            match item {
                Ok(SubjectMessage { source, data, .. }) => {
                    debug!(?data);

                    match data {
                        Message::SyncRequest => {
                            if let Some(source) = source {
                                peer.publish(
                                    &IdentTopic::new(format!("LWW-SOURCE-{source}")),
                                    &Message::Update {
                                        updates: vec![("ALL".into(), store.encode_all())],
                                    },
                                )
                                .await?;
                            }
                        }
                        Message::Update { updates } => {
                            for (_, update) in updates {
                                store.apply(&update);
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
}

impl Deref for LwwStore {
    type Target = EphemeralStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
