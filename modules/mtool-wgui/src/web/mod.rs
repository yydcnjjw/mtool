pub mod app;
mod auto_resize_window;
mod keybinding;
mod route;
mod switch;

use async_trait::async_trait;
use mapp::{define_label, provider::Res, AppContext, AppModule, ModuleGroup, ScheduleGraph};
use route::global_router;

pub use auto_resize_window::*;
pub use keybinding::*;
pub use route::{RouteParams, Router};

#[derive(Default)]
struct Module {}

define_label!(
    pub enum AppStage {
        Startup,
        Init,
        Run,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(global_router()));

        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![AppStage::Startup, AppStage::Init, AppStage::Run],
        );

        ctx.schedule().add_once_task(AppStage::Run, run);

        Ok(())
    }
}

async fn run() -> Result<(), anyhow::Error> {
    yew::Renderer::<app::App>::new().render();
    Ok(())
}

#[allow(unused)]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-wgui-web");

    group.add_module(Module::default());

    group
}
