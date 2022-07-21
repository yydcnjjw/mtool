mod daemon;
mod path;

use anyhow::Context;
use cmder_mod::Cmder;
use config_mod::Config;
use daemon::DaemonCmd;
use keybinding_mod::KeyBinding;
use mrpc::Server;
use mtool_service::*;
use std::{env, sync::Arc};
use sysev_mod::Sysev;
use tokio::sync::OnceCell;

struct App {
    cli: ServiceClient,

    sysev: OnceCell<Arc<Sysev>>,
    config: OnceCell<Arc<Config>>,
    keybinding: OnceCell<Arc<KeyBinding>>,
    cmder: OnceCell<Arc<Cmder>>,
}

impl App {
    async fn new(cli: ServiceClient) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            cli,
            sysev: OnceCell::const_new(),
            config: OnceCell::const_new(),
            keybinding: OnceCell::const_new(),
            cmder: OnceCell::const_new(),
        }))
    }

    async fn run_serve(
        rx: mrpc::sync::mpsc::Receiver<mrpc::message::Request<ServiceRequest, ServiceResponse>>,
        cli: ServiceClient,
    ) {
        match App::new(cli).await {
            Ok(app) => app.run_loop(rx).await,
            Err(e) => {
                log::error!("Failed to new App: {:?}", e);
            }
        }
    }

    async fn run() -> anyhow::Result<ServiceClient> {
        let (tx, rx) = mrpc::transport::native::channel(32);

        let cli = ServiceClient::new(Arc::new(tx));

        let serve = tokio::spawn(Self::run_serve(rx, cli.clone()));

        cli.cmder()
            .add("daemon".into(), DaemonCmd::new(serve))
            .await?;

        cmder_mod::load(cli.cmder()).await?;
        toast_mod::load(cli.cmder()).await?;
        translate_mod::load(cli.cmder(), cli.config()).await?;
        dict_mod::load(cli.cmder(), cli.config()).await?;

        anynav_mod::load(cli.keybinding(), cli.cmder()).await?;

        Ok(cli)
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

async fn run_cmd(cli: ServiceClient) -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let cmd = args.first().context("At least one parameter")?;

    cli.cmder().exec(cmd.clone(), args).await?;
    Ok(())
}

fn logger_init() -> anyhow::Result<()> {
    log4rs::init_file(
        path::logger_config_file().context("Failed to get logger config file")?,
        Default::default(),
    )?;
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    if let Err(e) = logger_init() {
        println!("{:?}", e);
        return Ok(());
    }

    run_cmd(App::run().await?).await
}

#[tokio::main]
async fn main() {
    
    if let Err(e) = run().await {
        log::error!("{:?}", e);
    }
}
