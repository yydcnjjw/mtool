use anyhow::Context;

use cmder_mod::Cmder;
use config_mod::Config;
use keybinding_mod::KeyBinding;
use mrpc::Server;
use mtool_service::*;
use std::sync::Arc;
use sysev_mod::Sysev;
use tokio::sync::OnceCell;

use crate::{path, Args};

pub struct App {
    args: Args,

    cli: ServiceClient,

    sysev: OnceCell<Arc<Sysev>>,
    config: OnceCell<Arc<Config>>,
    keybinding: OnceCell<Arc<KeyBinding>>,
    cmder: OnceCell<Arc<Cmder>>,
}

impl App {
    async fn new(args: Args, cli: ServiceClient) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            args,
            cli,
            sysev: OnceCell::const_new(),
            config: OnceCell::const_new(),
            keybinding: OnceCell::const_new(),
            cmder: OnceCell::const_new(),
        }))
    }

    async fn load_module(cli: ServiceClient) -> anyhow::Result<()> {
        // TODO: di
        cmder_mod::load(cli.cmder()).await?;
        toast_mod::load(cli.cmder()).await?;
        translate_mod::load(cli.cmder(), cli.config()).await?;
        dict_mod::load(cli.cmder(), cli.config()).await?;
        anynav_mod::load(cli.keybinding(), cli.cmder(), cli.config()).await?;
        Ok(())
    }

    pub async fn run(args: Args) -> anyhow::Result<()> {
        let (tx, rx) = mrpc::transport::native::channel(32);

        let cli = ServiceClient::new(Arc::new(tx));

        let app = App::new(args, cli.clone())
            .await
            .context("Failed to create app")?;

        let app_ = app.clone();
        let serve = tokio::spawn(async { app_.run_loop(rx).await });

        Self::load_module(cli.clone())
            .await
            .context("Failed to load module")?;

        if app.args.daemon {
            log::info!("mtool running at daemon");
            serve.await?;
        }

        Ok(())
    }
}

#[mrpc::service]
impl Service for App {
    async fn get_sysev(self: Arc<Self>) -> sysev_mod::SharedService {
        self.sysev
            .get_or_init(|| async move { Sysev::new() })
            .await
            .clone()
    }

    async fn get_config(self: Arc<Self>) -> config_mod::SharedService {
        self.config
            .get_or_init(|| async move {
                Config::new(
                    path::config_file()
                        .context("Failed to open savev file")
                        .unwrap(),
                )
                .await
            })
            .await
            .clone()
    }

    async fn get_keybinding(self: Arc<Self>) -> keybinding_mod::SharedService {
        let sysev = self.cli.sysev();
        self.keybinding
            .get_or_init(|| async move { KeyBinding::new(sysev).await.unwrap() })
            .await
            .clone()
    }

    async fn get_cmder(self: Arc<Self>) -> cmder_mod::SharedService {
        let keybinding = self.cli.keybinding();
        self.cmder
            .get_or_init(|| async move { Cmder::new(keybinding).await.unwrap() })
            .await
            .clone()
    }
}
