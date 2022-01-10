use anyhow::Context;
use mkeybinding::{KeyCombine, KeyDispatcher};
use msysev::KeyAction;
use std::sync::Arc;
use sysev_mod::ServiceClient as SysevCli;
use tokio::sync::{broadcast, RwLock};

use crate::{SerdeResult, Service};

pub struct KeyBinding {
    dispatcher: Arc<RwLock<KeyDispatcher<String>>>,
}

impl KeyBinding {
    pub async fn new<SysevPoster>(sysevcli: SysevCli<SysevPoster>) -> anyhow::Result<Arc<Self>>
    where
        SysevPoster: sysev_mod::ServicePoster,
    {
        let rx = sysevcli.subscribe().await?;

        let self_ = Arc::new(Self {
            dispatcher: Arc::new(RwLock::new(KeyDispatcher::new())),
        });

        tokio::spawn(Self::run_loop(self_.clone(), rx));

        Ok(self_)
    }

    async fn run_loop(self: Arc<Self>, mut rx: broadcast::Receiver<msysev::Event>) {
        while let Ok(e) = rx.recv().await {
            match e {
                msysev::Event::Key(e) if matches!(e.action, KeyAction::Press) => {
                    self.dispatcher.write().await.dispatch(KeyCombine {
                        key: e.keycode,
                        mods: e.modifiers,
                    });
                }
                _ => {}
            }
        }

        log::info!("keybinding loop exited");
    }
}

#[mrpc::async_trait]
impl Service for KeyBinding {
    async fn define_key_binding(self: Arc<Self>, kbd: String, cmd: String) -> SerdeResult<()> {
        self.dispatcher
            .write()
            .await
            .keymap()
            .add(&*kbd, cmd.clone())
            .context(format!("Failed to define key binding {} <-> {}", kbd, cmd))
            .map_err(|e| serde_error::Error::new(&*e))
    }

    async fn remove_key_binding(self: Arc<Self>, kbd: String) -> SerdeResult<()> {
        self.dispatcher
            .write()
            .await
            .keymap()
            .remove(&*kbd)
            .context(format!("Failed to remove key binding {}", kbd))
            .map_err(|e| serde_error::Error::new(&*e))
    }

    async fn subscribe(self: Arc<Self>) -> broadcast::Receiver<String> {
        self.dispatcher.read().await.subscribe()
    }
}
