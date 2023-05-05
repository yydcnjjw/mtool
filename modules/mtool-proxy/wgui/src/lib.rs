mod app;

use app::App;
use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule, ModuleGroup};
use mtool_wgui_core::{AppStage, RouteParams, Router};
use yew::prelude::*;

#[derive(Default)]
struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(AppStage::Startup, startup);
        Ok(())
    }
}

fn render(_: &RouteParams) -> Html {
    html! {
        <App/>
    }
}

async fn startup(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/proxy", render);
    Ok(())
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("proxy_group");

    group.add_module(Module::default());

    group
}
