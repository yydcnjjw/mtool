mod builder;

pub use builder::*;

use async_trait::async_trait;
use mapp::{
    define_label,
    provider::{Injector, Res, Take},
    AppContext, AppModule, ModuleGroup,
};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage, CmdlineStage,
};
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};
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
                is_startup_mode(StartupMode::Gui),
            )
            .add_once_task(GuiStage::Setup, setup)
            .add_once_task(GuiStage::Init, init)
            .add_once_task(AppStage::Run, wait_for_exit);

        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("gui_group");
    group.add_module(Module::default());
    group
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    let (tx, rx) = oneshot::channel();
    builder.setup(|builder| {
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
            .setup(move |app| {
                tx.send(Res::new(app.handle())).unwrap();
                Ok(())
            }))
    })?;

    injector.construct_once(|| async move { Ok(rx.await?) });
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

async fn wait_for_exit(worker: Take<TauriWorker>) -> Result<(), anyhow::Error> {
    worker.take()?.0.await?;
    Ok(())
}
