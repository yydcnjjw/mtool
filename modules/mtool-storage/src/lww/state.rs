use loro::{LoroBinaryValue, LoroValue};
use mapp::{
    anyhow::{self, anyhow},
    futures::future,
    prelude::*,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
    tokio::sync::watch,
    tracing::{debug, warn},
};
use std::{fmt::Debug, future::Future};

use super::LwwStore;

#[derive(Clone)]
pub struct State<T> {
    key: String,
    service: Res<LwwStore>,
    sender: watch::Sender<T>,
}

impl<T> State<T>
where
    T: Serialize + DeserializeOwned + Debug + Send + Sync + 'static,
{
    pub async fn new<K>(service: Res<LwwStore>, key: K) -> Result<Self, anyhow::Error>
    where
        K: ToString,
        T: Default,
    {
        Self::new_with(service, key, || future::ready(Ok(T::default()))).await
    }

    pub async fn new_with<K, F, O>(
        service: Res<LwwStore>,
        key: K,
        default: F,
    ) -> Result<Self, anyhow::Error>
    where
        K: ToString,
        F: FnOnce() -> O,
        O: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
    {
        let key = key.to_string();

        let value = if let Some(value) = service.get(&key) {
            State::value_from(value)?
        } else {
            let value = default().await?;
            debug!("state initialize default value: {:?}", value);
            value
        };

        let (sender, _) = watch::channel(value);
        {
            let sender = sender.clone();
            service.subscribe_value_changed(key.clone(), move |value| {
                match State::value_from(value.clone()) {
                    Ok(value) => {
                        sender.send_replace(value);
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
                sender.sender_count() == 1
            });
        }

        Ok(State {
            key,
            sender,
            service,
        })
    }

    pub fn set(&self, value: T) {
        match State::to_loro(&value) {
            Ok(value) => {
                self.service.set(&self.key, value);
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

    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    fn value_from(value: LoroValue) -> Result<T, anyhow::Error> {
        Ok(serde_json::from_slice(
            &value
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
