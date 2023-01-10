use async_trait::async_trait;
use mapp::{define_label, AppContext, AppModule, Label, ModuleGroup};

mod cmdline;
pub mod config;
pub mod logger;
mod test;

pub use cmdline::*;
pub use config::ConfigStore;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::default();

    group
        .add_module(CoreModule::default())
        .add_module(cmdline::Module::default())
        .add_module(config::Module::default())
        .add_module(logger::Module::default())
        .add_module(test::Module::default());

    group
}

define_label!(
    pub enum StartupStage {
        PreStartup,
        Startup,
        PostStartup,
    }
);

define_label!(
    pub enum InitStage {
        PreInit,
        Init,
        PostInit,
    }
);

define_label!(
    pub enum RunStage {
        PreRun,
        Run,
        PostRun,
    }
);

define_label!(
    pub enum ExitStage {
        PreExit,
        Exit,
        PostExit,
    }
);

define_label!(
    pub enum AppStage {
        Startup,
        Exit,
    }
);

#[derive(Default)]
struct CoreModule {}

#[async_trait]
impl AppModule for CoreModule {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_stage(AppStage::Startup)
            .await
            .insert_stage(AppStage::Startup, StartupStage::PreStartup)
            .await
            .insert_stage(StartupStage::PreStartup, StartupStage::Startup)
            .await
            .insert_stage(StartupStage::Startup, StartupStage::PostStartup)
            .await
            .insert_stage(StartupStage::PostStartup, InitStage::PreInit)
            .await
            .insert_stage(InitStage::PreInit, InitStage::Init)
            .await
            .insert_stage(InitStage::Init, InitStage::PostInit)
            .await
            .insert_stage(InitStage::PostInit, RunStage::PreRun)
            .await
            .insert_stage(RunStage::PreRun, RunStage::Run)
            .await
            .insert_stage(RunStage::Run, RunStage::PostRun)
            .await
            .insert_stage(RunStage::PostRun, ExitStage::PreExit)
            .await
            .insert_stage(ExitStage::PreExit, ExitStage::Exit)
            .await
            .insert_stage(ExitStage::Exit, ExitStage::PostExit)
            .await
            .insert_stage(ExitStage::PostExit, AppStage::Exit)
            .await;
        Ok(())
    }
}
