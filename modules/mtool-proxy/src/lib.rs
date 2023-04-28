mod cmd;
mod proxy;
mod wgui;

use async_trait::async_trait;
use cmd::add_proxy_rule;
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor, ModuleGroup};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{
    config::{not_startup_mode, StartupMode},
    AppStage, CmdlineStage,
};
use proxy::ProxyApp;
use tracing::log::warn;

#[derive(Default)]
pub struct Module {}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(add_proxy_rule.name("add_proxy_rule"));
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
        app.injector().construct_once(ProxyApp::construct);

        app.schedule()
            .add_once_task(
                CmdlineStage::AfterInit,
                register_command.cond(not_startup_mode(StartupMode::Cli)),
            )
            .add_once_task(AppStage::Run, run.cond(not_startup_mode(StartupMode::Cli)));
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
