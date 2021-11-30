mod event;

use std::{
    any::{Any, TypeId},
    ops::Deref,
    sync::Arc,
};

use anyhow::Context;
use cloud_api::tencent::api::ResponseType;
use futures::SinkExt;
use tokio::sync::{broadcast, oneshot};

pub use self::event::{Event, ResponsiveEvent};

pub type DynamicEvent = Arc<dyn Any + Send + Sync>;

pub type Sender = broadcast::Sender<DynamicEvent>;
pub type Receiver = broadcast::Receiver<DynamicEvent>;

pub type Responder<T> = oneshot::Sender<T>;

pub struct EventBus {
    tx: Sender,
}

impl EventBus {
    pub fn new(cap: usize) -> Self {
        let (tx, _) = broadcast::channel(cap);
        Self { tx }
    }

    pub fn sender(&self) -> &Sender {
        &self.tx
    }

    pub fn subscribe(&self) -> Receiver {
        self.tx.subscribe()
    }
}

pub async fn post_result<I, O>(sender: Sender, data: I) -> anyhow::Result<O>
where
    I: 'static + Send + Sync,
    O: 'static + Send + Sync,
{
    let (tx, rx) = oneshot::channel::<O>();
    sender
        .send(Arc::new(ResponsiveEvent::new(data, tx)))
        .context("Send event")?;
    Ok(rx.await.context("Wait result")?)
}

pub fn post<T>(sender: Sender, data: T)
where
    T: 'static + Send + Sync,
{
    sender.send(Arc::new(Event::new(data))).unwrap();
}
