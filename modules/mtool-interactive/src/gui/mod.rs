mod builder;
pub mod completion;
mod interactive_windows;
mod output;

pub use builder::*;
pub use completion::Completion;
pub use interactive_windows::*;
pub use output::OutputDevice;

use async_trait::async_trait;
use mapp::{
    define_label,
    provider::{Injector, Res},
    AppContext, AppModule, Label,
};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    CmdlineStage,
};
use std::vec;
use tokio::sync::oneshot;

#[derive(Default)]
pub struct Module {}

define_label! {
    pub enum GuiStage {
        Setup,
        Init,
        AfterInit,
    }
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Builder::new);

        app.schedule()
            .insert_stage_vec_with_cond(
                CmdlineStage::AfterInit,
                vec![GuiStage::Setup, GuiStage::Init, GuiStage::AfterInit],
                is_startup_mode(StartupMode::Gui),
            )
            .add_once_task(GuiStage::Setup, setup)
            .add_once_task(GuiStage::Init, init)
            .add_once_task(GuiStage::AfterInit, register_keybinding);

        Ok(())
    }
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    let (tx, rx) = oneshot::channel();
    builder.setup(|builder| {
        let injector = injector.clone();
        Ok(builder
            .setup(move |app| {
                tx.send(Res::new(app.handle())).unwrap();
                injector.insert(InteractiveWindow::new_inner(app.handle())?);
                Ok(())
            })
            .plugin(completion::init())
            .plugin(output::init()))
    })?;

    injector
        .construct_once(|| async move { Ok(rx.await?) })
        .construct_once(InteractiveWindow::new)
        .construct_once(Completion::new)
        .construct_once(OutputDevice::new);
    Ok(())
}

async fn init(builder: Res<Builder>) -> Result<(), anyhow::Error> {
    let builder = builder.take();

    tokio::task::spawn_blocking(move || {
        builder
            .any_thread()
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
    Ok(())
}

#[cfg(not(windows))]
use mtool_system::keybinding::Keybinging;

#[cfg(not(windows))]
async fn register_keybinding(keybinding: Res<Keybinging>) -> Result<(), anyhow::Error> {
    keybinding.define_global("M-A-q", interactive_windows::hide_window)?;
    Ok(())
}

#[cfg(windows)]
use tauri::AppHandle;

#[cfg(windows)]
async fn register_keybinding(app: Res<AppHandle>, injector: Injector) -> Result<(), anyhow::Error> {
    use mapp::inject::inject;
    use tauri::{async_runtime::spawn, GlobalShortcutManager};

    app.global_shortcut_manager()
        .register("Super+Alt+Q", move || {
            let injector = injector.clone();
            spawn(async move {
                if let Err(e) = inject(&injector, &interactive_windows::hide_window).await {
                    log::warn!("{}", e);
                }
            });
        })?;
    Ok(())
}
