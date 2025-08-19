mod receiver;
mod rpc;

use mapp::{anyhow, prelude::*};
use mtool_core::AppStage;
use mtool_rpc::{Router, RpcStage};
use receiver::NotifyReceiver;
use rpc::RpcService;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(NotifyReceiver::construct);
        ctx.schedule()
            .add_once_task(AppStage::Init, NotifyReceiver::listen_system)
            .add_once_task(RpcStage::Setup, setup);
        Ok(())
    }
}

async fn setup(router: Res<Router>, ctx: Res<NotifyReceiver>) -> Result<(), anyhow::Error> {
    router.add_service(RpcService::server(ctx)?);
    Ok(())
}
