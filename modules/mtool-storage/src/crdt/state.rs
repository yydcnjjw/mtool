use loro::{ContainerTrait, EventTriggerKind, LoroBinaryValue, LoroMap, LoroValue, Subscription};
use mapp::{
    anyhow::{self, anyhow},
    prelude::*,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
    tokio::sync::watch,
    tracing::{debug, warn},
};
use std::{borrow::Cow, fmt::Debug, future::Future, sync::Arc};

use super::CrdtService;

#[derive(Clone)]
pub struct State<T> {
    key: String,
    store: LoroMap,
    sender: watch::Sender<T>,
    _subscription: Arc<Option<Subscription>>,
}

impl<T> State<T>
where
    T: Serialize + DeserializeOwned + Debug + Send + Sync + 'static,
{
    pub async fn new_with<K, F, O>(
        service: Res<CrdtService>,
        key: K,
        default: F,
    ) -> Result<Self, anyhow::Error>
    where
        K: ToString,
        F: FnOnce() -> O,
        O: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
    {
        let store = service.store.get_map("STATE");

        let key = key.to_string();

        let value = if let Some(value) = store.get(&key) {
            State::value_from(value)?
        } else {
            let value = default().await?;
            debug!("state initialize default value: {:?}", value);

            store.insert(&key, Self::to_loro(&value)?)?;
            store.doc().map(|doc| doc.commit());
            value
        };

        let (sender, _) = watch::channel(value);
        let subscription = {
            let sender = sender.clone();
            let key = key.clone();
            store.subscribe(Arc::new(move |ev| {
                if let EventTriggerKind::Import = ev.triggered_by {
                    debug!("import {:?}", ev);
                    for ev in ev.events {
                        if let Some(Some(value)) = ev
                            .diff
                            .into_map()
                            .unwrap()
                            .updated
                            .get(&Cow::Borrowed(key.as_str()))
                        {
                            match State::value_from(value.clone()) {
                                Ok(value) => {
                                    sender.send_replace(value);
                                }
                                Err(e) => {
                                    warn!("{:?}", e);
                                }
                            }
                        }
                    }
                }
            }))
        };

        Ok(State {
            key,
            store,
            sender,
            _subscription: Arc::new(subscription),
        })
    }

    pub fn set(&self, value: T) {
        match State::to_loro(&value) {
            Ok(value) => {
                if let Err(e) = self.store.insert(&self.key, value) {
                    warn!("{:?}", e);
                } else {
                    self.store.doc().map(|doc| doc.commit());
                }
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
        self.sender.send_replace(value);
    }

    pub fn borrow(&self) -> watch::Ref<'_, T> {
        self.sender.borrow()
    }

    pub fn subscribe(&self) -> watch::Receiver<T> {
        self.sender.subscribe()
    }

    fn value_from(value: loro::ValueOrContainer) -> Result<T, anyhow::Error> {
        Ok(serde_json::from_slice(
            &value
                .into_value()
                .map_err(|e| anyhow!("failed to into_value: {:?}", e))?
                .into_binary()
                .map_err(|e| anyhow!("failed to into_binary: {:?}", e))?,
        )?)
    }

    fn to_loro(value: &T) -> Result<LoroValue, anyhow::Error> {
        Ok(LoroValue::Binary(LoroBinaryValue::from(
            serde_json::to_vec::<T>(&value)?,
        )))
    }
}
