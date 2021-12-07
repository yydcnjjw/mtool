use std::{
    env::{self, temp_dir},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

use crate::{
    core::{
        self,
        command::ExecCommand,
        evbus::{post, EventBus, Sender},
    },
    module,
};

fn get_default_tmp_path() -> std::path::PathBuf {
    temp_dir().join("mytool.log")
}

pub struct App {
    pub evbus: Arc<EventBus>,
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let evbus = Arc::new(EventBus::new(32));
        Ok(Self { evbus })
    }

    async fn exec_cmd(tx: Sender) -> anyhow::Result<()> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        if args.is_empty() {
            return Ok(());
        }

        let (cmd, args) = args.split_first().unwrap();
        if let Err(e) = ExecCommand::post(&tx, cmd.into(), args.to_vec()).await {
            log::error!("{}", e);
        }
        QuitApp::post(&tx, 0);

        Ok(())
    }

    fn logger_init() -> anyhow::Result<()> {
        let stdout = ConsoleAppender::builder().build();

        let requests = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(get_default_tmp_path())
            .context("Build file appender")?;

        let config = log4rs::config::Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("mytool", Box::new(requests)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("mytool")
                    .build(LevelFilter::Debug),
            )
            .context("Config builer failed")?;

        let _handle = log4rs::init_config(config).context("log4rs init config")?;
        Ok(())
    }

    pub async fn run_loop() -> anyhow::Result<()> {
        App::logger_init().context("logger init failed")?;

        let app = App::new().await?;

        core::module_load(&app).await?;
        module::module_load(&app).await?;

        let tx = app.evbus.sender();

        tokio::spawn(async move { App::exec_cmd(tx).await });

        while app.evbus.sender().receiver_count() != 0 {
            tokio::time::sleep(Duration::from_millis(300)).await;
        }

        Ok(())
    }
}

pub struct QuitApp {
    #[allow(dead_code)]
    ec: i32,
}

impl QuitApp {
    pub fn post(tx: &Sender, ec: i32) {
        if let Err(e) = post(tx, QuitApp { ec }) {
            log::error!("{}", e);
        }
    }
}
