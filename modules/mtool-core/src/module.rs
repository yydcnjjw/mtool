use mapp::{anyhow, define_label, prelude::*};

use crate::{config, logger};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("core");

    group.add_module(CoreModule);

    #[cfg(feature = "cmdline")]
    group.add_module(crate::cmdline::Module);

    group.add_module(config::Module).add_module(logger::Module);

    group
}

struct CoreModule;

define_label!(
    pub enum AppStage {
        Startup,
        BeforeInit,
        Init,
        AfterInit,
        Run,
    }
);

#[async_trait]
impl AppModule for CoreModule {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![
                AppStage::Startup,
                AppStage::BeforeInit,
                AppStage::Init,
                AppStage::AfterInit,
                AppStage::Run,
            ],
        );
        Ok(())
    }
}
