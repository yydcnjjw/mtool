use clap::{arg, ArgMatches};
use mapp::{anyhow, prelude::*, tokio, tracing::warn, CreateOnceTaskDescriptor};
use mtool_cmdpal::CommandPalette;
use mtool_core::{AppStage, Cmdline, CmdlineStage, ConfigStore};
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
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(DioxusStage::Setup, dioxus_setup.cond(is_runnable))
            .add_once_task(AppStage::Run, run_proxy_service.cond(is_runnable));
        Ok(())
    }
}

pub async fn is_runnable(
    config: Res<ConfigStore>,
    args: Res<ArgMatches>,
) -> Result<bool, anyhow::Error> {
    Ok(!args.get_flag("no-proxy"))
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline.setup(|cmdline| Ok(cmdline.arg(arg!(--"no-proxy" "no proxy"))))
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
