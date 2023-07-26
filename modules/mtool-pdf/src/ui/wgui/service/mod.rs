mod pdf_render;

use async_trait::async_trait;
use mapp::{prelude::*, CreateOnceTaskDescriptor};
use mtool_core::config::{is_startup_mode, StartupMode};
use mtool_wgui::{Builder, WGuiStage};
use tracing::warn;

use crate::pdf::Pdf;

use self::pdf_render::pdf_protocol_handler;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(
            WGuiStage::Setup,
            setup.cond(is_startup_mode(StartupMode::WGui)),
        );
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, pdf: Res<Pdf>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| {
        Ok(builder
            .plugin(pdf_render::init(pdf))
            .register_uri_scheme_protocol("pdfviewer", |handle, req| {
                Ok(pdf_protocol_handler(handle, req).inspect_err(|e| warn!("{:?}", e))?)
            }))
    })
}
