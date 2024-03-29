use async_trait::async_trait;
use mapp::{define_label, AppContext, AppModule, ModuleGroup, ScheduleGraph};

mod cmdline;
pub mod config;
pub mod logger;
mod test;

pub use cmdline::*;
pub use config::ConfigStore;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("core_group");

    group
        .add_module(CoreModule::default())
        .add_module(cmdline::Module::default())
        .add_module(config::Module::default())
        .add_module(logger::Module::default())
        .add_module(test::Module::default());

    group
}

#[derive(Default)]
struct CoreModule {}

define_label!(
    pub enum AppStage {
        Startup,
        Init,
        Run,
    }
);

#[async_trait]
impl AppModule for CoreModule {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![AppStage::Startup, AppStage::Init, AppStage::Run],
        );
        Ok(())
    }
}
