mod geosite_item;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod cmd;
        mod service;
        pub use service::*;

        use clap::{arg, ArgMatches};
        use mtool_core::{config::StartupMode, AppStage, Cmdline, CmdlineStage, ConfigStore};
        use tracing::warn;

    }
}

use async_trait::async_trait;
use mapp::prelude::*;

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, app: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        use geosite_item::GeositeItemView;
        use mtool_wgui::{Templator, WebStage};
        app.schedule()
            .add_once_task(WebStage::Init, |templator: Res<Templator>| async move {
                templator.add_template::<GeositeItemView>();
                Ok::<(), anyhow::Error>(())
            });
        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        use mapp::CreateOnceTaskDescriptor;
        app.injector().construct_once(ProxyService::construct);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(CmdlineStage::AfterInit, cmd::register.cond(is_runnable))
            .add_once_task(AppStage::Run, run.cond(is_runnable));

        async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
            cmdline.setup(|cmdline| Ok(cmdline.arg(arg!(--"without-proxy" "without proxy"))))
        }

        async fn run(app: Res<ProxyService>) -> Result<(), anyhow::Error> {
            tokio::spawn(async move {
                if let Err(e) = app.run().await {
                    warn!("proxy is exited: {:?}", e);
                }
            });
            Ok(())
        }
        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
pub async fn is_runnable(
    config: Res<ConfigStore>,
    args: Res<ArgMatches>,
) -> Result<bool, anyhow::Error> {
    Ok(config.startup_mode() != StartupMode::Cli && !args.get_flag("without-proxy"))
}
