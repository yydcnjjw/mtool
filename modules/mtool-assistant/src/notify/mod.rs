mod context;
mod media;

use mapp::{anyhow, prelude::*};
use mtool_core::AppStage;

pub use context::*;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(NotifyContext::construct);
        ctx.schedule()
            .add_once_task(AppStage::Init, NotifyContext::init);
        Ok(())
    }
}
