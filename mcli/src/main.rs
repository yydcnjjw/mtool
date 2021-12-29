use std::{env, path::PathBuf, sync::Arc};

use anyhow::Context;

use cmder_mod::Cmder;
use config_mod::{self, Config};
use keybinding_mod::KeyBinding;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
};
use sysev_mod::{self, Sysev};

use tokio::sync::mpsc;

#[mrpc::server]
enum Server {
    Sysev(sysev_mod::Service),
    Config(config_mod::Service),
    Keybinding(keybinding_mod::Service),
    Cmder(cmder_mod::Service),
}

struct App {
    cli: ServerClient,
}

impl App {
    async fn new(cli: ServerClient) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self { cli }))
    }

    async fn run_serve(
        rx: mpsc::Receiver<mrpc::Message<ServerRequest, ServerResponse>>,
        cli: ServerClient,
    ) {
        match App::new(cli).await {
            Ok(app) => {
                if let Err(e) = app.serve(rx).await {
                    log::error!("App exited with error: {:?}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to new App: {:?}", e);
            }
        }
    }

    async fn run() -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel(32);
        let cli = ServerClient { sender: tx };

        let _serve = tokio::spawn(Self::run_serve(rx, cli.clone()));

        toast_mod::load(cli.cmder()).await?;
        translate_mod::load(cli.cmder(), cli.config()).await?;

        let args: Vec<String> = env::args().skip(1).collect();

        let cmd = args.first().context("At least one parameter")?;

        cli.cmder().exec(cmd.clone(), args).await?;

        Ok(())
    }
}

#[mrpc::async_trait]
impl Server for App {
    async fn create_sysev(self: Arc<Self>) -> anyhow::Result<Arc<dyn sysev_mod::Service>> {
        Ok(Sysev::new())
    }
    async fn create_config(self: Arc<Self>) -> anyhow::Result<Arc<dyn config_mod::Service>> {
        Ok(Config::new(default_config_path().context("Failed to open config file")?).await)
    }
    async fn create_keybinding(
        self: Arc<Self>,
    ) -> anyhow::Result<Arc<dyn keybinding_mod::Service>> {
        Ok(KeyBinding::new(self.cli.sysev()).await?)
    }

    async fn create_cmder(self: Arc<Self>) -> anyhow::Result<Arc<dyn cmder_mod::Service>> {
        Ok(Cmder::new(self.cli.keybinding()).await?)
    }
}

fn logger_init() -> anyhow::Result<()> {
    let stdout = ConsoleAppender::builder().build();

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .context("Failed to build logger config")?;

    let _handle = log4rs::init_config(config).context("Failed to init log4rs config")?;
    Ok(())
}

fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".my-tool").join("config.toml"))
}

#[tokio::main]
async fn main() {
    logger_init().unwrap();

    if let Err(e) = App::run().await {
        log::error!("{:?}", e);
    }
}
