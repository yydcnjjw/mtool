mod app;
mod component;
mod event;
mod route;

use app::App;
use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_wgui::{AppStage, RouteParams, Router};
use yew::prelude::*;

#[allow(unused)]
#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(AppStage::Init, init);
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
