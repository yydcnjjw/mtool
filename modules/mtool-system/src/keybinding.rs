use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use mapp::{AppContext, AppModule, CreateTaskDescriptor, Res};
use mkeybinding::{KeyCombine, KeyDispatcher};
use msysev::{Event, KeyAction};
use tokio::sync::{broadcast, RwLock};

use mtool_core::{config::is_daemon, InitStage};

use super::event;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct(Keybinging::new).await;

        app.schedule()
            .add_task(InitStage::Init, init.cond(is_daemon))
            .await;
        Ok(())
    }
}

struct Keybinging {
    dispatcher: Arc<RwLock<KeyDispatcher<String>>>,
}

impl Keybinging {
    async fn new(ob: Res<event::Observer>) -> Result<Res<Self>, anyhow::Error> {
        let dispatcher = Arc::new(RwLock::new(KeyDispatcher::new()));

        let mut rx = ob.subscribe();

        let dispatcher_c = dispatcher.clone();
        tokio::spawn(async move {
            while let Ok(e) = rx.recv().await {
                match e {
                    Event::Key(e) if matches!(e.action, KeyAction::Press) => {
                        log::trace!("{:?}", e);

                        dispatcher_c.write().await.dispatch(KeyCombine {
                            key: e.keycode,
                            mods: e.modifiers,
                        });
                    }
                    _ => {}
                }
            }
        });

        Ok(Res::new(Self { dispatcher }))
    }

    pub async fn define_key_binding(&self, kbd: &str, cmd: String) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .await
            .keymap()
            .add(kbd, cmd.clone())
            .context(format!("Failed to define key binding {} <-> {}", kbd, cmd))
    }

    #[allow(unused)]
    pub async fn remove_key_binding(&self, kbd: &str) -> Result<(), anyhow::Error> {
        self.dispatcher
            .write()
            .await
            .keymap()
            .remove(kbd)
            .context(format!("Failed to remove key binding {}", kbd))
    }

    pub async fn subscribe(&self) -> broadcast::Receiver<String> {
        self.dispatcher.read().await.subscribe()
    }
}

async fn init(keybinging: Res<Keybinging>) -> Result<(), anyhow::Error> {
    keybinging
        .define_key_binding("C-c c", "test".into())
        .await?;
    let mut ob = keybinging.subscribe().await;

    tokio::spawn(async move {
        while let Ok(s) = ob.recv().await {
            log::info!("task: {}", s);
        }
    });
    Ok(())
}
