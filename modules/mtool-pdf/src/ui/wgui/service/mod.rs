mod error;
// mod grobid;
mod pdf_document;
mod pdf_loader;

use async_trait::async_trait;
use mapp::{prelude::*, CreateOnceTaskDescriptor};
use mtool_core::config::{is_startup_mode, StartupMode};
use mtool_wgui::{Builder, WGuiStage};

pub use error::Error;
pub use pdf_document::*;
pub use pdf_loader::*;

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

async fn setup(builder: Res<Builder>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(pdf_loader::init())))
}
