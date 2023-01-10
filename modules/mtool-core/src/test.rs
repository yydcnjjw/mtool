use async_trait::async_trait;
use mapp::{AppContext, AppModule};

use crate::CmdlineStage;

#[derive(Default)]
pub struct Module {}

async fn run() -> Result<(), anyhow::Error> {
    Ok(())
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(CmdlineStage::AfterInit, run);
        Ok(())
    }
}
