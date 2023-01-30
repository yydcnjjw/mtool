use std::{future::Future, sync::Arc};

use crate::{KeyCombine, KeyDispatcher};
use anyhow::Context;
use async_trait::async_trait;
use msysev::{Event, KeyAction};
use tokio::sync::{broadcast::Receiver, RwLock};

#[async_trait]
pub trait Action {
    async fn do_action(&self) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl<Func, Output> Action for Func
where
    Func: Fn() -> Output + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    async fn do_action(&self) -> Result<(), anyhow::Error> {
        (self)().await
    }
}

type SharedAction = Arc<dyn Action + Send + Sync>;

type Dispatcher = Arc<RwLock<KeyDispatcher<SharedAction>>>;

pub struct Keybinging {
    dispatcher: Dispatcher,
}

impl Keybinging {
    pub fn new(rx: Receiver<Event>) -> Result<Self, anyhow::Error> {
        let dispatcher = Arc::new(RwLock::new(KeyDispatcher::new()));

        tokio::spawn(Self::run_loop(dispatcher.clone(), rx));

        Ok(Self { dispatcher })
    }

    async fn run_loop(dispatcher: Dispatcher, mut rx: Receiver<Event>) {
        while let Ok(e) = rx.recv().await {
            match e {
                Event::Key(e) if matches!(e.action, KeyAction::Press) => {
                    log::trace!("{:?}", e);

                    dispatcher.write().await.dispatch(KeyCombine {
                        key: e.keycode,
                        mods: e.modifiers,
                    });
                }
                _ => {}
            }
        }
    }

    pub async fn define<T>(&self, kbd: &str, action: T) -> Result<(), anyhow::Error>
    where
        T: Action + Send + Sync + 'static,
    {
        self.define_raw(kbd, Arc::new(action)).await
    }

    async fn define_raw(&self, kbd: &str, action: SharedAction) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .await
            .keymap()
            .add(kbd, action)
            .context(format!("Failed to define key binding {}", kbd))
    }

    pub async fn remove(&self, kbd: &str) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .await
            .keymap()
            .remove(kbd)
            .context(format!("Failed to remove key binding {}", kbd))
    }
}
