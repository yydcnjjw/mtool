mod service;

use mapp::{anyhow, prelude::*};

use mtool_rpc::{Router, RpcStage};
use service::RpcService;

pub use service::pb::{mtool_assistant_client::MtoolAssistantClient, *};

use crate::notify::NotifyContext;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(RpcStage::Setup, setup);
        Ok(())
    }
}

async fn setup(router: Res<Router>, ctx: Res<NotifyContext>) -> Result<(), anyhow::Error> {
    router.add_service(RpcService::server(ctx)?);
    Ok(())
}
