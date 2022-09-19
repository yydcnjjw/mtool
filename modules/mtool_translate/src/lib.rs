mod cmd;
mod language;
mod tencent;
mod translator;

use async_trait::async_trait;
use mapp::{AppContext, AppModule, CreateTaskDescriptor, Res};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{config::is_cli, InitStage};

#[derive(Default)]
pub struct Module {}

async fn init_for_cli(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(cmd::te.name("te"))
        .await
        .add_command(cmd::tz.name("tz"))
        .await
        .add_command(cmd::tj.name("tj"))
        .await;
    Ok(())
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct(tencent::Translator::new).await;

        app.schedule()
            .add_task(InitStage::Init, init_for_cli.cond(is_cli))
            .await;
        Ok(())
    }
}
