mod geosite_item;

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        use mtool_wgui::Templator;
        use geosite_item::GeositeItemView;
    } else {
        mod cmd;
        mod service;
        pub use service::*;

        use clap::{arg, ArgMatches};
        use mapp::CreateOnceTaskDescriptor;
        use mtool_core::{config::StartupMode, AppStage, Cmdline, CmdlineStage, ConfigStore};
        use tracing::warn;

    }
}

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        #[cfg(target_family = "wasm")]
        app.schedule().add_once_task(
            mtool_wgui::AppStage::Init,
            |templator: Res<Templator>| async move {
                templator.add_template::<GeositeItemView>();
                Ok::<(), anyhow::Error>(())
            },
        );

        #[cfg(not(target_family = "wasm"))]
        {
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
                })
                .await?;
                Ok(())
            }
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
