use std::ops::Deref;

use crate::{
    app::App,
    core::{
        evbus::{Event, EventBus, Receiver, Sender},
        service::Service,
    },
};

use sysev;
use tokio::sync::broadcast;

use super::{kbdispatcher::KeyBindingDispatcher, Result};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub kbd: String,
    pub cmd_name: String,
}

pub struct KeyBindinger {}

impl KeyBindinger {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_loop(mut rx: Receiver) {
        log::info!("KeyBindinger is running");
        while let Ok(e) = rx.recv().await {
            if let Some(e) = e.downcast_ref::<Event<KeyBinding>>() {
                log::info!("{:?}", e.deref());
            }
        }
        log::info!("KeyBindinger finished");
    }
}
