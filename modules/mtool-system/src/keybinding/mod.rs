mod action;

#[cfg(target_os = "linux")]
mod sysev_backend;

#[cfg(target_os = "linux")]
mod dbus_backend;

// #[cfg(windows)]
// mod windows_backend;

use std::{future::Future, sync::Arc};

use dashmap::DashMap;
use mapp::{
    define_label,
    prelude::{inject::*, *},
};
use mkeybinding::KeySequence;
use mtool_core::config::{is_wayland, is_x11};
use tokio::sync::mpsc;

use action::{FnAction, SharedAction};
use tracing::{debug, warn};

#[derive(Default)]
pub struct Module {}

pub fn module() -> ModuleGroup {
    #[allow(unused_mut)]
    let mut group = ModuleGroup::new("keybinding_group");

    #[cfg(target_os = "linux")]
    {
        if is_wayland() {
            group.add_module(dbus_backend::Module);
        }

        if is_x11() {
            group.add_module(sysev_backend::Module);
        }
    }

    // #[cfg(windows)]
    // group.add_module(windows_backend::Module::default());

    group
}

pub struct Keybinding {
    kbs: DashMap<KeySequence, SharedAction>,

    hotkey_mgr: Res<dyn SetupGlobalHotKey + Send + Sync>,
}

impl Keybinding {
    pub fn new<T>(hotkey_mgr: Res<T>) -> Self
    where
        T: SetupGlobalHotKey + Send + Sync + 'static,
    {
        Self {
            kbs: DashMap::new(),
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

        self.kbs.insert(ks.clone(), Arc::new(FnAction::new(action)));

        self.hotkey_mgr.register(&ks).await
    }

    pub async fn remove_global(&self, kbd: &str) -> Result<(), anyhow::Error> {
        debug!("remove global keybinding {}", kbd);

        let ks = KeySequence::parse(kbd)?;
        self.kbs.remove(&ks);
        self.hotkey_mgr.unregister(&ks).await
    }

    pub async fn run(
        self: Res<Keybinding>,
        injector: Injector,
        mut rx: mpsc::UnboundedReceiver<GlobalHotKeyEvent>,
    ) {
        while let Some(ev) = { rx.recv().await } {
            debug!("handle action {}", ev.0.to_string());
            if let Some(action) = self.kbs.get(&ev.0).map(|v| v.clone()) {
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
