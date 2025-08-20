use dioxus::prelude::*;
use mapp::{anyhow, prelude::*};
use mtool_rpc::{Router, RpcStage};

use super::{AnyPlayer, PlayerService};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(PlayerService::construct);
        ctx.schedule().add_once_task(RpcStage::Setup, setup);
        Ok(())
    }
}

async fn setup(router: Res<Router>, service: Res<PlayerService>) -> Result<(), anyhow::Error> {
    router.add_service(PlayerService::server(service)?);
    Ok(())
}
