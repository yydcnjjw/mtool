mod app;
mod pdf;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_wgui::{AppStage, RouteParams, Router};
use yew::prelude::*;

use self::{app::App, pdf::Pdf};

pub struct Module;

#[cfg(target_family = "wasm")]
#[async_trait(?Send)]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(Pdf::construct);
        ctx.schedule().add_once_task(AppStage::Init, init);
        Ok(())
    }
}

fn render(_: &RouteParams, pdf: Res<Pdf>) -> Html {
    html! {
        <App {pdf}/>
    }
}

async fn init(router: Res<Router>, pdf: Res<Pdf>) -> Result<(), anyhow::Error> {
    router.add("/pdfviewer", move |params| render(params, pdf.clone()));
    Ok(())
}
