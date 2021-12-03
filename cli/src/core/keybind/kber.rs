use crate::{
    app::App,
    core::{
        evbus::{EventBus, Receiver, Sender},
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

    pub async fn define_key_binding(&mut self, kbd: &str, cmd: &str) -> Result<()> {
        self.kb_dispatcher
            .add_keybinding(KeyBinding {
                kbd: kbd.to_string(),
                cmd_name: cmd.to_string(),
            })
            .await
    }

    async fn recv_loop(mut rx: broadcast::Receiver<KeyBinding>) {
        loop {
            match rx.recv().await {
                Ok(kb) => log::info!("{:?}", kb),
                Err(e) => {
                    log::error!("{}", e);
                    break;
                }
            }
        }
    }

    pub async fn run_loop(tx: Sender, mut rx: Receiver) {
        // let kber = KeyBindinger::new();

        KeyBindingDispatcher::run_loop(tx, rx);
    }
}

#[async_trait]
impl Service for KeyBindinger {
    async fn run_loop(&mut self) {
        let rx = self.kb_dispatcher.subscribe();

        tokio::spawn(KeyBindinger::recv_loop(rx));

        self.kb_dispatcher.run_loop().await;
    }
}

pub async fn define_globale_key(app: &mut App, kbd: &str, cmd: &str) -> Result<()> {
    app.kber.define_key_binding(kbd, cmd).await
}
