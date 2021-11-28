use std::{env, path::PathBuf};

use anyhow::Context;
use futures::future::join_all;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use mytool_core::config::Config;

use crate::{core::{command::Commander, evbus::EventBus, keybind::KeyBindinger}, module::module_load};

pub struct App {
    pub cfg: Config,
    pub cmder: Commander,
    pub evbus: EventBus,
    pub kber: KeyBindinger,
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".my-tool").join("config.toml"))
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let cfg = Config::load(config_path().context("Get config path")?).await?;
        let cmder = Commander::new();
        let evbus = EventBus::new(10);
        let kber = KeyBindinger::new(&evbus);

        Ok(Self {
            cfg,
            cmder,
            evbus,
            kber,
        })
    }

    async fn exec_cmd(&mut self) -> anyhow::Result<()> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        let (cmd, args) = args.split_first().unwrap();

        self.cmder.exec(cmd, args).await
    }

    fn logger_init() {
        let stdout = ConsoleAppender::builder().build();

        let requests = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build("/tmp/mytool.log")
            .unwrap();

        let config = log4rs::config::Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("mytool", Box::new(requests)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("mytool")
                    .build(LevelFilter::Debug),
            )
            .unwrap();

        let handle = log4rs::init_config(config).unwrap();
    }

    async fn run_sysev_loop(&self) {

        .await;
    }

    pub async fn run() -> anyhow::Result<()> {
        App::logger_init();

        let mut app = App::new().await?;

        module_load(&mut app).await?;

        app.exec_cmd().await?;

        log::debug!("Run service !!!");

        let j1 = app.run_sysev_loop();

        let j2 = app.kber.run_loop();

        join_all(vec![j1, j2]).await;

        Ok(())
    }
}
