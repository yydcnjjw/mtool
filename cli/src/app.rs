use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use async_trait::async_trait;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use mytool_core::config::Config;

use crate::core::{
    command::{AddCommand, Command},
    evbus::EventBus,
    keybind::DefineKeyBinding,
    module_load,
};

pub struct App {
    pub cfg: Config,
    pub evbus: Arc<EventBus>,
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".my-tool").join("config.toml"))
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let cfg = Config::load(config_path().context("Get config path")?).await?;
        let evbus = Arc::new(EventBus::new(32));

        Ok(Self {
            cfg,
            evbus,
        })
    }

    #[allow(dead_code)]
    async fn exec_cmd(&mut self) -> anyhow::Result<()> {
        // let args = env::args().skip(1).collect::<Vec<String>>();
        // let (cmd, args) = args.split_first().unwrap();

        // self.cmder.exec(cmd, args).await
        Ok(())
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

        let _handle = log4rs::init_config(config).unwrap();
    }

    pub async fn run_loop() -> anyhow::Result<()> {
        App::logger_init();

        let app = App::new().await?;

        module_load(&app).await?;

        let sender = &app.evbus.sender();
        DefineKeyBinding::post(sender, "C-m a", "test").await??;
        DefineKeyBinding::post(sender, "C-m c", "test").await??;

        AddCommand::post(sender, "test".into(), TestCmd {}).await?;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

struct TestCmd {}
#[async_trait]
impl Command for TestCmd {
    async fn exec(&mut self, _args: Vec<String>) -> anyhow::Result<()> {
        println!("test");
        Ok(())
    }
}
