mod app;
mod auto_window;
pub mod component;
mod keybinding;
mod route;
mod switch;
mod template;

pub use app::*;
pub use auto_window::*;
pub use keybinding::*;
pub use route::*;
pub use template::{EmptyView, Template, TemplateData, TemplateId, TemplateView, Templator};

use mapp::{
    anyhow, define_label,
    futures::{future::BoxFuture, FutureExt},
    prelude::*,
};
use mtauri_sys::prelude::*;

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
        ctx.injector()
            .insert(Res::new(Keybinding::new_with_window()));

        ctx.schedule().insert_stage_vec(
            ScheduleGraph::Root,
            vec![WebStage::Startup, WebStage::Init, WebStage::Run],
        );

        ctx.schedule().add_once_task(WebStage::Run, run);
        Ok(())
    }
}

async fn run(keybinding: Res<Keybinding>, templator: Res<Templator>) -> Result<(), anyhow::Error> {
    yew::Renderer::<WebApp>::with_props(WebAppContext {
        keybinding,
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

pub fn is_window(
    label: &'static str,
) -> impl Fn() -> BoxFuture<'static, Result<bool, anyhow::Error>> + Clone {
    move || async move { Ok(Window::current()?.label() == label) }.boxed()
}

pub fn contains_window_label(
    label: &'static str,
) -> impl Fn() -> BoxFuture<'static, Result<bool, anyhow::Error>> + Clone {
    move || async move { Ok(Window::current()?.label().contains(label)) }.boxed()
}
