mod collins;
mod mdx;

use async_trait::async_trait;
use mapp::{AppContext, AppModule, CreateTaskDescriptor, Res};
use mdx::mdx_query;
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{config::is_cli, InitStage};

#[derive(Default)]
pub struct Module {}

async fn init_for_cli(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(mdx_query.name("mdx"))
        .await
        .add_command(collins::dict.name("dict").add_alias("d"))
        .await
        .add_command(collins::thesaures.name("thesaures").add_alias("dt"))
        .await;
    Ok(())
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_task(InitStage::Init, init_for_cli.cond(is_cli))
            .await;
        Ok(())
    }
}
