use std::{
    future::Future,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use mapp::{
    inject::{inject, Inject, Provide},
    provider::{Injector, Res},
    AppContext, AppModule, CreateOnceTaskDescriptor,
};
use mkeybinding::{KeyCombine, KeyDispatcher, KeyMap};
use msysev::{Event, KeyAction};
use mtool_core::{
    config::{not_startup_mode, StartupMode},
    AppStage,
};

use tokio::sync::broadcast::Receiver;

use super::event;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once({
            let injector = app.injector().clone();
            move || Keybinging::new(injector)
        });

        app.schedule().add_once_task(
            AppStage::Run,
            Keybinging::run.cond(not_startup_mode(StartupMode::Cli)),
        );
        Ok(())
    }
}

#[async_trait]
pub trait Action<C> {
    async fn do_action(&self, c: &C) -> Result<(), anyhow::Error>;
}

pub struct FnAction<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnAction<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, C> Action<C> for FnAction<Func, Args>
where
    Func: Inject<Args> + Send + Sync,
    Func::Output: Future<Output = Result<(), anyhow::Error>> + Send,
    Args: Provide<C> + Send + Sync,
    C: Send + Sync,
{
    async fn do_action(&self, c: &C) -> Result<(), anyhow::Error> {
        inject(c, &self.f).await?.await
    }
}

type SharedAction = Arc<dyn Action<Injector> + Send + Sync>;

pub struct Keybinging {
    dispatcher: RwLock<KeyDispatcher<SharedAction>>,
    injector: Injector,
}

impl Keybinging {
    const GLOBAL_KEYMAP: &str = "global";

    async fn new(injector: Injector) -> Result<Res<Self>, anyhow::Error> {
        let mut dispatcher = KeyDispatcher::new();

        dispatcher.push_keymap(Self::GLOBAL_KEYMAP, KeyMap::new());

        Ok(Res::new(Self {
            dispatcher: RwLock::new(dispatcher),
            injector,
        }))
    }

    async fn run(self_: Res<Self>, ob: Res<event::Observer>) -> Result<(), anyhow::Error> {
        let rx = ob.subscribe();
        tokio::spawn(Self::run_loop(self_.clone(), rx));
        tokio::spawn(Self::run_action_loop(self_.clone()));
        Ok(())
    }

    async fn run_loop(self_: Res<Self>, mut rx: Receiver<Event>) {
        while let Ok(e) = rx.recv().await {
            match e {
                Event::Key(e) if matches!(e.action, KeyAction::Press) => {
                    self_.dispatcher.write().unwrap().dispatch(KeyCombine {
                        key: e.keycode,
                        mods: e.modifiers,
                    });
                }
                _ => {}
            }
        }
    }

    async fn run_action_loop(self_: Res<Self>) {
        let mut rx = self_.dispatcher.read().unwrap().subscribe();
        while let Ok(action) = rx.recv().await {
            let injector = self_.injector.clone();
            tokio::spawn(async move {
                if let Err(e) = action.do_action(&injector).await {
                    log::warn!("Failed to do action: {}", e);
                }
            });
        }
    }
}

#[cfg(not(windows))]
use anyhow::Context;

#[cfg(not(windows))]
impl Keybinging {
    pub async fn define_global<Args, T>(&self, kbd: &str, action: T) -> Result<(), anyhow::Error>
    where
        T: Inject<Args> + Send + Sync + 'static,
        T::Output: Future<Output = Result<(), anyhow::Error>> + Send,
        Args: Provide<Injector> + Send + Sync + 'static,
    {
        self.define_global_raw(kbd, Arc::new(FnAction::new(action)))
    }

    fn define_raw(&self, km: &str, kbd: &str, action: SharedAction) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .unwrap()
            .get_keymap_mut(km)
            .unwrap()
            .add(kbd, action)
            .context(format!("Failed to define key binding {}", kbd))
    }

    fn define_global_raw(&self, kbd: &str, action: SharedAction) -> Result<(), anyhow::Error> {
        self.define_raw(Self::GLOBAL_KEYMAP, kbd, action)
    }

    pub fn remove_global(&self, kbd: &str) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .unwrap()
            .get_keymap_mut(Self::GLOBAL_KEYMAP)
            .unwrap()
            .remove(kbd)
            .context(format!("Failed to remove key binding {}", kbd))
    }
}

#[cfg(windows)]
impl Keybinging {
    pub async fn define_global<Args, T>(&self, kbd: &str, action: T) -> Result<(), anyhow::Error>
    where
        T: Inject<Args> + Send + Sync + 'static,
        T::Output: Future<Output = Result<(), anyhow::Error>> + Send,
        Args: Provide<Injector> + Send + Sync + 'static,
    {
        use tauri::{async_runtime::spawn, AppHandle, GlobalShortcutManager};

        let app = self.injector.get::<Res<AppHandle>>().await?;
        let injector = self.injector.clone();
        let action = Arc::new(action);
        app.global_shortcut_manager().register(kbd, move || {
            let injector = injector.clone();
            let action = action.clone();
            spawn(async move {
                let result = match inject(&injector, action.as_ref()).await {
                    Ok(v) => v,
                    Err(e) => {
                        return log::warn!("Failed to inject: {}", e);
                    }
                };

                if let Err(e) = result.await {
                    log::warn!("Failed to do action: {}", e);
                }
            });
        })?;
        Ok(())
    }
}