mod cmd;
mod proxy;
mod wgui;

use async_trait::async_trait;
use clap::{arg, ArgMatches};
use cmd::{add_proxy_rule, remove_proxy_rule};
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor, ModuleGroup};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{config::StartupMode, AppStage, Cmdline, CmdlineStage, ConfigStore};
use proxy::ProxyApp;
use tracing::log::warn;

#[derive(Default)]
pub struct Module {}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(add_proxy_rule.name("add_proxy_rule"))
        .add_command(
            remove_proxy_rule
                .name("remove_proxy_rule")
                .desc("remove proxy rule from file"),
        );

    Ok(())
}

pub(crate) async fn run(app: Res<ProxyApp>) -> Result<(), anyhow::Error> {
    tokio::spawn(async move {
        if let Err(e) = app.run().await {
            warn!("proxy is exited: {:?}", e);
        }
    })
    .await?;
    Ok(())
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline.setup(|cmdline| Ok(cmdline.arg(arg!(--"without-proxy" "without proxy"))))
}

async fn is_runnable(config: Res<ConfigStore>, args: Res<ArgMatches>) -> Result<bool, anyhow::Error> {
    Ok(config.startup_mode() != StartupMode::Cli && !args.get_flag("without-proxy"))
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(ProxyApp::construct);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(CmdlineStage::AfterInit, register_command.cond(is_runnable))
            .add_once_task(AppStage::Run, run.cond(is_runnable));
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("proxy_group");
    group
        .add_module(Module::default())
        .add_module(wgui::Module::default());
    group
}
