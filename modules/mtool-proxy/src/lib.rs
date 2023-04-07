mod cmd;
mod proxy;

use async_trait::async_trait;
use cmd::add_proxy_target;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{AppStage, CmdlineStage};
use proxy::ProxyApp;
use tracing::log::warn;

#[derive(Default)]
pub struct Module {}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(add_proxy_target.name("add_proxy_target"));
    Ok(())
}

async fn run(app: Res<ProxyApp>) -> Result<(), anyhow::Error> {
    tokio::spawn(async move {
        if let Err(e) = app.run().await {
            warn!("proxy is exited: {:?}", e);
        }
    })
    .await?;
    Ok(())
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(ProxyApp::new);

        app.schedule()
            .add_once_task(CmdlineStage::AfterInit, register_command)
            .add_once_task(AppStage::Run, run);
        Ok(())
    }
}
