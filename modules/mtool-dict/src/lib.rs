mod collins;
mod mdx;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mdx::mdx_query;
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::CmdlineStage;

#[derive(Default)]
pub struct Module {}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(mdx_query.name("mdx"))
        .add_command(collins::dict.name("dict").add_alias("d"))
        .add_command(collins::thesaures.name("thesaures").add_alias("dt"));
    Ok(())
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(CmdlineStage::AfterInit, register_command);
        Ok(())
    }
}
