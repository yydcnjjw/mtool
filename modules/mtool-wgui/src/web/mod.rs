mod app;
mod auto_resize_window;
mod component;
mod keybinding;
mod route;
mod switch;
mod template;

use async_trait::async_trait;
use mapp::{
    define_label, provider::Res, AppContext as AppCtx, AppModule, ModuleGroup, ScheduleGraph,
};

pub use app::*;
pub use auto_resize_window::*;
pub use keybinding::*;
pub use route::*;
pub use template::{EmptyView, Template, TemplateData, TemplateId, TemplateView, Templator};

struct Module;

define_label!(
    pub enum AppStage {
        Startup,
        Init,
        Run,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppCtx) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(global_router()));

        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![AppStage::Startup, AppStage::Init, AppStage::Run],
        );

        ctx.schedule().add_once_task(AppStage::Run, run);
        Ok(())
    }
}

async fn run(templator: Res<Templator>) -> Result<(), anyhow::Error> {
    yew::Renderer::<app::App>::with_props(app::AppContext {
        keybinding: Keybinding::new_with_window(),
        templator,
    })
    .render();
    Ok(())
}

#[allow(unused)]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-wgui-web");

    group.add_module(Module);
    group.add_module(template::Module);

    group
}
