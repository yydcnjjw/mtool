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

struct App {
    cli: ServiceClient,
}

impl App {
    async fn new(cli: ServiceClient) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self { cli }))
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
        webterm_mod::load().await?;

        Ok(cli)
    }
}

#[mrpc::service]
impl Service for App {
    async fn create_sysev(self: Arc<Self>) -> anyhow::Result<sysev_mod::SharedService> {
        Ok(Sysev::new())
    }
    async fn create_config(self: Arc<Self>) -> anyhow::Result<config_mod::SharedService> {
        Ok(Config::new(path::config_file().context("Failed to open config file")?).await)
    }
    async fn create_keybinding(self: Arc<Self>) -> anyhow::Result<keybinding_mod::SharedService> {
        Ok(KeyBinding::new(self.cli.sysev()).await?)
    }

    async fn create_cmder(self: Arc<Self>) -> anyhow::Result<cmder_mod::SharedService> {
        Ok(Cmder::new(self.cli.keybinding()).await?)
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
