mod collins;
mod mdx;

use anyhow::Context;
use async_trait::async_trait;
use mapp::{AppContext, AppModule, CreateTaskDescriptor, Res};
use mdx::{mdx, MdictConfig};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{config::is_cli, Config, InitStage};
use serde::Deserialize;

#[derive(Default)]
pub struct Module {}

async fn init_for_cli(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(mdx.name("mdx"))
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
        app.injector().construct(DictConfig::new).await;

        app.schedule()
            .add_task(InitStage::Init, init_for_cli.cond(is_cli))
            .await;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DictConfig {
    mdict: MdictConfig,
}

impl DictConfig {
    pub async fn new(config: Res<Config>) -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(
            config
                .get::<Self>("dict")
                .await
                .context("Failed to parse dict.mdict")?,
        ))
    }
}
