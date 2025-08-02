use mapp::{anyhow, prelude::*};
use mtool_core::AppStage;
use mtool_dioxus::prelude::*;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, setup_dioxus)
            .add_once_task(AppStage::Init, init);
        Ok(())
    }
}

async fn setup_dioxus() -> Result<(), anyhow::Error> {
    Ok(())
}

async fn init() -> Result<(), anyhow::Error> {
    Ok(())
}
