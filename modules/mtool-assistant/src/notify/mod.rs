mod receiver;

use mapp::{anyhow, prelude::*};
use mtool_core::AppStage;
use receiver::NotifyReceiver;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(NotifyReceiver::construct);
        ctx.schedule()
            .add_once_task(AppStage::Init, NotifyReceiver::listen_system);
        Ok(())
    }
}
