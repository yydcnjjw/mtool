use std::{
    any::{Any, TypeId},
    ops::Add,
    sync::Arc,
};

use anyhow::Context;
use futures::future::join_all;
use tokio::sync::oneshot;

use crate::{
    app::App,
    core::evbus::{self, post_result, Event, EventBus, Receiver, ResponsiveEvent, Sender},
};

use super::{DynamicService, Service};

pub struct AddService {
    pub service: DynamicService,
}

impl AddService {
    pub async fn post(sender: &Sender, service: DynamicService) -> anyhow::Result<()> {
        Ok(post_result::<AddService, ()>(sender, AddService { service }).await?)
    }
}

pub struct RunAll {}

impl RunAll {
    pub async fn post(sender: &Sender) -> anyhow::Result<()> {
        Ok(post_result::<RunAll, ()>(sender, RunAll {}).await?)
    }
}

pub struct Server {
    services: Vec<DynamicService>,
}

impl Server {
    fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    fn add_service(&mut self, service: DynamicService) {
        self.services.push(service)
    }

    async fn run_all(&mut self) {
        join_all(self.services.iter_mut().map(|s| s.run_loop())).await;
    }

    pub async fn run_loop(mut rx: Receiver) {
        let mut server = Server::new();

        while let Ok(e) = rx.recv().await {
            if let Some(e) = e.downcast_ref::<ResponsiveEvent<AddService, ()>>() {
                server.add_service(e.service.clone());
                e.result(());
            } else if let Some(e) = e.downcast_ref::<ResponsiveEvent<RunAll, ()>>() {
                server.run_all().await;
                e.result(());
                break;
            }
        }
    }
}
