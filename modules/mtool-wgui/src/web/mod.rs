mod app;
mod auto_window;
pub mod component;
mod keybinding;
mod route;
mod switch;
mod template;

use async_trait::async_trait;
use mapp::{define_label, prelude::*, ScheduleGraph};

pub use app::*;
pub use auto_window::*;
pub use keybinding::*;
pub use route::*;
pub use template::{EmptyView, Template, TemplateData, TemplateId, TemplateView, Templator};

struct Module;

define_label!(
    pub enum WebStage {
        Startup,
        Init,
        Run,
    }
);

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(global_router()));

        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![WebStage::Startup, WebStage::Init, WebStage::Run],
        );

        ctx.schedule().add_once_task(WebStage::Run, run);
        Ok(())
    }
}

async fn run(templator: Res<Templator>) -> Result<(), anyhow::Error> {
    yew::Renderer::<WebApp>::with_props(WebAppContext {
        keybinding: Keybinding::new_with_window(),
        templator,
    })
    .render();
    Ok(())
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-wgui-web");
    group.add_module(Module);
    group.add_module(template::Module);
    group
}
