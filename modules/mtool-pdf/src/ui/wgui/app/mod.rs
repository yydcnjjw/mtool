mod app;

use anyhow::Context;
use async_trait::async_trait;
use base64::prelude::*;
use mapp::prelude::*;
use mtool_wgui::{component::error::render_result_view, RouteParams, Router, WebStage};
use yew::prelude::*;

use self::app::App;

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(WebStage::Init, init);
        Ok(())
    }
}

fn render(params: &RouteParams) -> Result<Html, anyhow::Error> {
    let path =
        String::from_utf8(BASE64_STANDARD.decode(params.get("path").context("path not exist")?)?)?;

    Ok(html! {
        <App {path}/>
    })
}

fn index(_params: &RouteParams) -> Result<Html, anyhow::Error> {
    Ok(html! {
        <div>{"Pdf Viewer"}</div>
    })
}

async fn init(router: Res<Router>) -> Result<(), anyhow::Error> {
    router
        .add("/pdfviewer/:path", |params| {
            render_result_view(render(params))
        })
        .add("/pdfviewer", |params| render_result_view(index(params)));
    Ok(())
}
