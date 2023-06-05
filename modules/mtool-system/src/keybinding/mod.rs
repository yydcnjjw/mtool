mod action;

#[cfg(not(windows))]
mod sysev_backend;

// #[cfg(windows)]
// mod windows_backend;

use std::{collections::HashMap, future::Future, sync::Arc};

use async_trait::async_trait;
use mapp::{
    inject::{Inject, Provide},
    provider::{Injector, Res},
    ModuleGroup,
};
use mkeybinding::KeySequence;
use tokio::sync::{mpsc, Mutex, RwLock};

use action::{FnAction, SharedAction};
use tracing::{debug, warn};

#[derive(Default)]
pub struct Module {}

pub fn module() -> ModuleGroup {
    #[allow(unused_mut)]
    let mut group = ModuleGroup::new("keybinding_group");

    #[cfg(not(windows))]
    group.add_module(sysev_backend::Module::default());

    // #[cfg(windows)]
    // group.add_module(windows_backend::Module::default());

    group
}

pub struct Keybinding {
    kbs: RwLock<HashMap<KeySequence, SharedAction>>,
    rx: Mutex<mpsc::UnboundedReceiver<GlobalHotKeyEvent>>,
    hotkey_mgr: Res<dyn SetGlobalHotKey + Send + Sync>,
}

impl Keybinding {
    pub fn new<T>(hotkey_mgr: Res<T>, rx: mpsc::UnboundedReceiver<GlobalHotKeyEvent>) -> Self
    where
        T: SetGlobalHotKey + Send + Sync + 'static,
    {
        Self {
            kbs: RwLock::new(HashMap::new()),
            rx: Mutex::new(rx),
            hotkey_mgr,
        }
    }
}

impl Keybinding {
    pub async fn define_global<Args, T>(&self, kbd: &str, action: T) -> Result<(), anyhow::Error>
    where
        T: Inject<Args> + Send + Sync + 'static,
        T::Output: Future<Output = Result<(), anyhow::Error>> + Send,
        Args: Provide<Injector> + Send + Sync + 'static,
    {
        debug!("define global keybinding {}", kbd);

        let ks = KeySequence::parse(kbd)?;

        self.kbs
            .write()
            .await
            .insert(ks.clone(), Arc::new(FnAction::new(action)));

        self.hotkey_mgr.register(&ks).await
    }

    pub async fn remove_global(&self, kbd: &str) -> Result<(), anyhow::Error> {
        debug!("remove global keybinding {}", kbd);

        let ks = KeySequence::parse(kbd)?;
        self.kbs.write().await.remove(&ks);
        self.hotkey_mgr.unregister(&ks).await
    }

    pub async fn handle_event_loop(self: Res<Keybinding>, injector: Injector) {
        while let Some(ev) = { self.rx.lock().await.recv().await } {
            debug!("handle action {}", ev.0.to_string());
            if let Some(action) = { self.kbs.read().await.get(&ev.0).cloned() } {
                let injector = injector.clone();
                tokio::spawn(async move {
                    if let Err(e) = action.do_action(&injector).await {
                        warn!("do {} action failed: {:?}", ev.0.to_string(), e);
                    }
                });
            }
        }

        debug!("global key event loop is exited");
    }
}

#[derive(Debug)]
pub struct GlobalHotKeyEvent(pub KeySequence);

#[async_trait]
pub trait SetGlobalHotKey {
    async fn register(&self, ks: &KeySequence) -> Result<(), anyhow::Error>;
    async fn unregister(&self, ks: &KeySequence) -> Result<(), anyhow::Error>;
}
