use std::{
    any::{Any, TypeId},
    ops::Add,
};

use futures::future::join_all;
use tokio::sync::oneshot;

use crate::core::evbus::{Event, EventBus, Receiver, ResponsiveEvent};

use super::Service;

pub struct Server {
    services: Vec<Box<dyn Service>>,
    source: Receiver,
}

struct AddService {
    pub service: Box<dyn Service>,
}

impl Server {
    pub fn new(evbus: &EventBus) -> Self {
        Self {
            services: Vec::new(),
            source: evbus.subscribe(),
        }
    }

    fn add_service(&mut self, service: Box<dyn Service>) {
        self.services.push(service)
    }

    async fn run(&mut self) {
        while let Some(e) = self.source.recv().await {
            match e.type_id() {
                TypeId::of::<ResponsiveEvent<AddService>>() => {
                    let e = e.downcast_ref::<ResponsiveEvent<AddService>>().unwrap();
                    self.add_service(e.service);
                }
            }
        }

        // join_all(self.services.iter_mut().map(|s| s.run_loop())).await;
    }
}
