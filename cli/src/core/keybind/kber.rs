use crate::{app::App, core::service::Service};

use sysev::{self, EventBus};
use tokio::sync::broadcast;

use super::{kbdispatcher::KeyBindingDispatcher, Result};

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub kbd: String,
    pub cmd_name: String,
}

// impl KeyBinding {
//     pub fn new(kbd: String, cmd_name: String) -> Self {
//         Self { kbd, cmd_name }
//     }
// }

pub struct KeyBindinger {
    kb_dispatcher: KeyBindingDispatcher,
}

impl KeyBindinger {
    pub fn new(evbus: &EventBus) -> Self {
        Self {
            kb_dispatcher: KeyBindingDispatcher::new(evbus),
        }
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
        log::debug!("Run recv Loop !!!");
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
}

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
