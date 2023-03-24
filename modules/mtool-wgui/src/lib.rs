mod builder;

pub use builder::*;

use async_trait::async_trait;
use mapp::{
    define_label,
    provider::{Injector, Res},
    AppContext, AppModule, ModuleGroup,
};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage, CmdlineStage,
};
use tokio::sync::oneshot;

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
            .add_once_task(AppStage::Run, run);

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
        Ok(builder.setup(move |app| {
            tx.send(Res::new(app.handle())).unwrap();
            Ok(())
        }))
    })?;

    injector.construct_once(|| async move { Ok(rx.await?) });
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

async fn run() -> Result<(), anyhow::Error> {
    Ok(())
}
