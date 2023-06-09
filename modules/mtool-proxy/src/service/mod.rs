mod cmd;
mod service;

pub use service::*;

use async_trait::async_trait;
use clap::{arg, ArgMatches};
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor};
use mtool_core::{config::StartupMode, AppStage, Cmdline, CmdlineStage, ConfigStore};
use tracing::warn;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(ProxyService::construct);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(CmdlineStage::AfterInit, cmd::register.cond(is_runnable))
            .add_once_task(AppStage::Run, run.cond(is_runnable));
        Ok(())
    }
}

pub async fn is_runnable(
    config: Res<ConfigStore>,
    args: Res<ArgMatches>,
) -> Result<bool, anyhow::Error> {
    Ok(config.startup_mode() != StartupMode::Cli && !args.get_flag("without-proxy"))
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline.setup(|cmdline| Ok(cmdline.arg(arg!(--"without-proxy" "without proxy"))))
}

pub(crate) async fn run(app: Res<ProxyService>) -> Result<(), anyhow::Error> {
    tokio::spawn(async move {
        if let Err(e) = app.run().await {
            warn!("proxy is exited: {:?}", e);
        }
    })
    .await?;
    Ok(())
}
