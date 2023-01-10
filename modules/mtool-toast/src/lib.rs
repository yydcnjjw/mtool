mod toast;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::CmdlineStage;
use toast::toast;

#[derive(Default)]
pub struct Module {}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(toast.name("toast"));
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
