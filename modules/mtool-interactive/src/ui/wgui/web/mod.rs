mod app;
mod component;
mod event;
mod route;

use app::App;
use async_trait::async_trait;
use mapp::prelude::*;
use mtool_wgui::{RouteParams, Router, WebStage};
use yew::prelude::*;

#[allow(unused)]
pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(WebStage::Init, init);
        Ok(())
    }
}

fn render(_: &RouteParams) -> Html {
    html! {
        <App/>
    }
}

async fn init(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/interactive/*", render);
    router.add("/interactive", render);
    Ok(())
}
