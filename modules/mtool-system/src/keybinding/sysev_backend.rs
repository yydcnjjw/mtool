use std::sync::RwLock;

use anyhow::Context;

use async_trait::async_trait;
use mapp::{
    provider::Res,
    AppContext, AppModule, CreateOnceTaskDescriptor,
};
use mkeybinding::{KeyCombine, KeyDispatcher, KeyMap, KeySequence};
use msysev::{Event, KeyAction};

use mtool_core::{
    config::{not_startup_mode, StartupMode},
    AppStage,
};
use tokio::sync::{broadcast::Receiver, mpsc};
use tracing::warn;

use crate::event;

use super::{GlobalHotKeyEvent, Keybinding, SetGlobalHotKey};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(GlobalHotKeyMgr::construct);

        app.schedule().add_once_task(
            AppStage::Run,
            GlobalHotKeyMgr::run.cond(not_startup_mode(StartupMode::Cli)),
        );
        Ok(())
    }
}

pub struct GlobalHotKeyMgr {
    dispatcher: RwLock<KeyDispatcher<KeySequence>>,
    sender: mpsc::UnboundedSender<GlobalHotKeyEvent>,
}

impl GlobalHotKeyMgr {
    const GLOBAL_KEYMAP: &str = "global";

    async fn construct() -> Result<Res<Keybinding>, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let this = Res::new(Self::new(tx));

        Ok(Res::new(Keybinding::new(this, rx)))
    }

    fn new(sender: mpsc::UnboundedSender<GlobalHotKeyEvent>) -> Self {
        let mut dispatcher = KeyDispatcher::new();

        dispatcher.push_keymap(Self::GLOBAL_KEYMAP, KeyMap::new());

        Self {
            dispatcher: RwLock::new(dispatcher),
            sender,
        }
    }

    async fn run(this: Res<Self>, ob: Res<event::Observer>) -> Result<(), anyhow::Error> {
        let ob = ob.subscribe();
        {
            let this = this.clone();
            tokio::spawn(async move { this.dispatch_key_loop(ob).await });
        }

        tokio::spawn(async move { this.handle_hotkey_loop().await });

        Ok(())
    }

    async fn dispatch_key_loop(&self, mut rx: Receiver<Event>) {
        while let Ok(e) = rx.recv().await {
            match e {
                Event::Key(e) if matches!(e.action, KeyAction::Press) => {
                    self.dispatcher.write().unwrap().dispatch(KeyCombine {
                        key: e.keycode,
                        mods: e.modifiers,
                    });
                }
                _ => {}
            }
        }
    }

    async fn handle_hotkey_loop(&self) {
        let mut rx = self.dispatcher.read().unwrap().subscribe();
        while let Ok(ks) = rx.recv().await {
            if let Err(e) = self.sender.send(GlobalHotKeyEvent(ks)) {
                warn!("send global hotkey event failed: {}", e);
            }
        }
    }
}

impl GlobalHotKeyMgr {
    fn define_raw(&self, km: &str, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .unwrap()
            .get_keymap_mut(km)
            .unwrap()
            .add(ks, ks.clone())
            .context(format!("Failed to define key binding {}", ks))
    }

    fn define_global_raw(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.define_raw(Self::GLOBAL_KEYMAP, ks)
    }
}

#[async_trait]
impl SetGlobalHotKey for GlobalHotKeyMgr {
    async fn register(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.define_global_raw(ks)
    }

    async fn unregister(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .unwrap()
            .get_keymap_mut(Self::GLOBAL_KEYMAP)
            .unwrap()
            .remove(ks)
            .context(format!("Failed to remove key binding {}", ks))
    }
}
