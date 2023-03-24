mod app;
mod route;
mod switch;

use async_trait::async_trait;
use mapp::{define_label, provider::Res, AppContext, AppModule, ModuleGroup, ScheduleGraph};
use route::global_router;

pub use route::{Router, RouteParams};

#[derive(Default)]
struct CoreModule {}

define_label!(
    pub enum AppStage {
        Startup,
        Run,
        Exit,
    }
);

#[async_trait]
impl AppModule for CoreModule {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(global_router()));

        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![AppStage::Startup, AppStage::Run, AppStage::Exit],
        );

        ctx.schedule().add_once_task(AppStage::Run, run);

        Ok(())
    }
}

pub async fn run() -> Result<(), anyhow::Error> {
    yew::Renderer::<app::App>::new().render();
    Ok(())
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("core_group");

    group.add_module(CoreModule::default());

    group
}
