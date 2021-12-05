use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use super::Responder;

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

pub struct ResponsiveEvent<I, O> {
    base: Event<I>,
    responder: Arc<Mutex<Option<Responder<O>>>>,
}

impl<I, O> ResponsiveEvent<I, O> {
    pub fn new(data: I, responder: Responder<O>) -> Self {
        Self {
            base: Event::new(data),
            responder: Arc::new(Mutex::new(Some(responder))),
        }
    }

    pub fn result(&self, o: O) {
        let mut tx = self.responder.lock().unwrap();
        let tx = tx.deref_mut();
        let tx = std::mem::replace(tx, None);

        if let Err(_) = tx.unwrap().send(o) {
            log::error!("Responsive event result send failed");
        }
    }
}

impl<I, O> Deref for ResponsiveEvent<I, O> {
    type Target = Event<I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
