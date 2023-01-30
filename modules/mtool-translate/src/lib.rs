mod cmd;
mod language;
mod tencent;
mod translator;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::CmdlineStage;

#[derive(Default)]
pub struct Module {}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(cmd::te.name("te"))
        .add_command(cmd::tz.name("tz"))
        .add_command(cmd::tj.name("tj"));
    Ok(())
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(tencent::Translator::new);

        app.schedule()
            .add_once_task(CmdlineStage::AfterInit, register_command);
        Ok(())
    }
}
