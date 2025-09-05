mod error;
mod pdf_document;
mod pdf_loader;

use mapp::{anyhow, prelude::*};
use mtool_core::ConfigStore;
use mtool_wgui::{Builder, WGuiStage};

#[allow(unused)]
pub use error::Error;
pub use pdf_document::*;
pub use pdf_loader::*;
use sea_orm::DatabaseConnection;

use crate::{pdf::PdfApi, Config};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(WGuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(
    builder: Res<Builder>,
    cs: Res<ConfigStore>,
    db: Res<DatabaseConnection>,
    pdf_api: Res<PdfApi>,
) -> Result<(), anyhow::Error> {
    let config: Config = cs.get("pdf")?;
    builder.setup(|builder| Ok(builder.plugin(pdf_loader::init(config, db, pdf_api))))
}
