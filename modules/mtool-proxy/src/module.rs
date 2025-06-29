use mapp::{anyhow, prelude::*, tokio, tracing::warn};
use mtool_cmdpal::CommandPalette;
use mtool_core::AppStage;
use mtool_dioxus::prelude::{DioxusBuilder, DioxusStage};

use crate::{cmdpal_commands, proxy_service::ProxyService};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-proxy");
    group.add_module(Module);
    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(ProxyService::construct);
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, dioxus_setup)
            .add_once_task(AppStage::Run, run_proxy_service);
        Ok(())
    }
}

async fn dioxus_setup(
    builder: Res<DioxusBuilder>,
    service: Res<ProxyService>,
    cmdpal: Res<CommandPalette>,
) -> Result<(), anyhow::Error> {
    builder.with_launch_builder(|builder| builder.with_context(service));
    cmdpal_commands::register(cmdpal).await?;
    Ok(())
}

async fn run_proxy_service(service: Res<ProxyService>) -> Result<(), anyhow::Error> {
    tokio::spawn(async move {
        if let Err(e) = service.run().await {
            warn!("{:?}", e);
        }
    });
    Ok(())
}
