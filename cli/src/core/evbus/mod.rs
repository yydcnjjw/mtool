use std::{
    any::{Any, TypeId},
    ops::Deref,
};

use cloud_api::tencent::api::ResponseType;
use futures::SinkExt;
use tokio::sync::{broadcast, oneshot};

type DynamicEvent = Box<dyn Any>;
pub type Sender = broadcast::Sender<DynamicEvent>;
pub type Receiver = broadcast::Receiver<DynamicEvent>;

type Responder<T> = oneshot::Sender<T>;

pub struct Event<T> {
    data: T,
}

impl<T> Event<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Deref for Event<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct ResponsiveEvent<T> {
    base: Event<T>,
    responder: Responder<T>,
}

impl<T> ResponsiveEvent<T> {
    pub fn new(data: T, responder: Responder<T>) -> Self {
        Self {
            base: Event::new(data),
            responder,
        }
    }

    fn result(&self, v: T) -> Result<(), T> {
        self.responder.send(v)
    }
}

impl<T> Deref for ResponsiveEvent<T> {
    type Target = Event<T>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub struct EventBus {
    tx: Sender,
}

impl EventBus {
    pub fn new(cap: usize) -> Self {
        let (tx, _) = broadcast::channel(cap);
        Self { tx }
    }

    pub fn sender(&self) -> Sender {
        self.tx.clone()
    }

    pub fn subscribe(&self) -> Receiver {
        self.tx.subscribe()
    }
}

pub async fn post_result<T: Send>(sender: &Sender, data: T) -> anyhow::Result<T> {
    let (tx, rx) = oneshot::channel();
    sender.send(Box::new(ResponsiveEvent::new(data, tx)))?;
    Ok(rx.await?)
}

pub async fn post<T: Send>(sender: &Sender, data: T) {
    sender.send(Box::new(Event::new(data)))
}
