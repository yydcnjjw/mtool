mod action;

#[cfg(target_os = "linux")]
mod sysev_backend;

#[cfg(target_os = "linux")]
mod dbus_backend;

// #[cfg(windows)]
// mod windows_backend;

use std::{collections::HashMap, future::Future, sync::Arc};

use mapp::{
    anyhow, define_label,
    prelude::*,
    tokio::{
        self,
        sync::{mpsc, Mutex, RwLock},
    },
    tracing::{debug, warn},
};

use action::{FnAction, SharedAction};
use mkeybinding::KeySequence;

#[derive(Default)]
pub struct Module {}

pub fn module() -> ModuleGroup {
    #[allow(unused_mut)]
    let mut group = ModuleGroup::new("keybinding_group");

    // #[cfg(not(windows))]
    // group.add_module(sysev_backend::Module);

    #[cfg(target_os = "linux")]
    group.add_module(dbus_backend::Module);

    // #[cfg(windows)]
    // group.add_module(windows_backend::Module::default());

    group
}

pub struct Keybinding {
    kbs: RwLock<HashMap<KeySequence, SharedAction>>,
    rx: Mutex<mpsc::UnboundedReceiver<GlobalHotKeyEvent>>,
    hotkey_mgr: Box<dyn SetupGlobalHotKey + Send + Sync>,
}

impl Keybinding {
    pub fn new<T>(hotkey_mgr: Res<T>, rx: mpsc::UnboundedReceiver<GlobalHotKeyEvent>) -> Self
    where
        T: SetupGlobalHotKey + Send + Sync + 'static,
    {
        Self {
            kbs: RwLock::new(HashMap::new()),
            rx: Mutex::new(rx),
            hotkey_mgr: Box::new(Res::try_unwrap(hotkey_mgr).unwrap()),
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

    pub async fn handle_event_loop(self_: Res<Keybinding>, injector: Injector) {
        while let Some(ev) = { self_.rx.lock().await.recv().await } {
            debug!("handle action {}", ev.0.to_string());
            if let Some(action) = { self_.kbs.read().await.get(&ev.0).cloned() } {
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
pub trait SetupGlobalHotKey {
    async fn register(&self, ks: &KeySequence) -> Result<(), anyhow::Error>;
    async fn unregister(&self, ks: &KeySequence) -> Result<(), anyhow::Error>;
}

define_label!(
    pub enum GlobalHotKeyStage {
        Register,
        Setup,
        UnRegister,
    }
);
