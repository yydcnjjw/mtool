mod runner;

use mapp::{
    anyhow,
    prelude::*,
};
use mtool_dioxus::prelude::*;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(DioxusStage::Setup, setup);
        Ok(())
    }
}

async fn setup(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("runner", runner::view);
    router.route("runner");
    Ok(())
}
