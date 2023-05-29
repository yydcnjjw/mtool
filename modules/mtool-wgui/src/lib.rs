mod builder;
mod global_hotkey;
mod window;

pub use builder::*;
pub use window::{MtoolWindow, WGuiWindow};

use async_trait::async_trait;
use mapp::{
    define_label,
    provider::{Injector, Res, Take, TakeOpt},
    AppContext, AppModule, CreateOnceTaskDescriptor, ModuleGroup,
};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage, CmdlineStage,
};
use mtool_system::keybinding::Keybinding;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tokio::sync::oneshot;
use tracing::{info, warn};

define_label! {
    pub enum GuiStage {
        Setup,
        Init,
        AfterInit,
    }
}

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Builder::new);

        app.schedule()
            .insert_stage_vec_with_cond(
                CmdlineStage::AfterInit,
                vec![GuiStage::Setup, GuiStage::Init, GuiStage::AfterInit],
                is_startup_mode(StartupMode::WGui),
            )
            .add_once_task(GuiStage::Setup, setup)
            .add_once_task(GuiStage::Init, init)
            .add_once_task(
                AppStage::Init,
                register_keybinding.cond(is_startup_mode(StartupMode::WGui)),
            )
            .add_once_task(AppStage::Run, wait_for_exit);

        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("wgui_group");
    group.add_module(Module::default());
    // #[cfg(windows)]
    // group.add_module(global_hotkey::Module::default());
    group
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    let (tx, rx) = oneshot::channel();

    injector.construct_once(|| async move { Ok(rx.await?) });

    builder.setup(move |builder| {
        let quit = CustomMenuItem::new("quit".to_string(), "Quit");
        let tray_menu = SystemTrayMenu::new().add_item(quit);

        let tray = SystemTray::new().with_menu(tray_menu);

        Ok(builder
            .system_tray(tray)
            .on_system_tray_event(|_, event| match event {
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                },
                _ => {}
            })
            .plugin(tauri_plugin_window::init())
            .plugin(window::init(injector))
            .setup(move |app| {
                tx.send(Res::new(app.handle())).unwrap();
                Ok(())
            }))
    })?;

    Ok(())
}

struct TauriWorker(tokio::task::JoinHandle<()>);

async fn init(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    let builder = builder.take();

    let worker = tokio::task::spawn_blocking(move || {
        match builder.any_thread().run(tauri::generate_context!()) {
            Ok(_) => {
                info!("tauri run loop is exited");
            }
            Err(e) => {
                warn!("tauri run loop is exited: {:?}", e);
            }
        }
    });

    injector.insert(Take::new(TauriWorker(worker)));

    Ok(())
}

async fn wait_for_exit(worker: TakeOpt<TauriWorker>) -> Result<(), anyhow::Error> {
    if let Some(worker) = worker.unwrap() {
        worker.take()?.0.await?;
    }

    Ok(())
}

async fn register_keybinding(keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
    keybinding
        .define_global("M-A-o", window::show_window)
        .await?;
    keybinding
        .define_global("M-A-S-o", window::hide_window)
        .await?;
    Ok(())
}
