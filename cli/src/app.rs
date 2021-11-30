use std::{env, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use async_trait::async_trait;
use futures::future::join_all;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use mytool_core::config::Config;

use crate::core::{evbus::{EventBus, post}, module::Module, service::{self, AddService, RunAll, Service}};

pub struct App {
    pub cfg: Config,
    // pub cmder: Commander,
    pub evbus: Arc<EventBus>,
    // pub kber: KeyBindinger,
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".my-tool").join("config.toml"))
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let cfg = Config::load(config_path().context("Get config path")?).await?;
        // let cmder = Commander::new();
        let evbus = Arc::new(EventBus::new(32));
        // let kber = KeyBindinger::new(&evbus);

        Ok(Self {
            cfg,
            // cmder,
            evbus,
            // kber,
        })
    }

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

        let handle = log4rs::init_config(config).unwrap();
    }

    async fn run_sysev_loop(&self) {

        // .await;
    }

    pub async fn run() -> anyhow::Result<()> {
        App::logger_init();

        let app = App::new().await?;

        service::module_load(&app).await?;

        let sender = app.evbus.sender();

        struct DemoService {}
        #[async_trait]
        impl Service for DemoService {
            async fn run_loop(&self) {
                println!("Demo Service is running");
            }
        }

        // tokio::time::sleep(Duration::from_millis(1000)).await;

        // let mut rx = app.evbus.subscribe();
        // tokio::spawn(async move {
        //     while let Ok(e) = rx.recv().await {
        //         println!("{:?}", e);
        //     }
        // });

        
        // post(sender.clone(), ());

        AddService::post(sender.clone(), Arc::new(DemoService {})).await?;
        
        RunAll::post(sender.clone()).await?;

        // module_load(&mut app).await?;

        // app.exec_cmd().await?;

        // log::debug!("Run service !!!");

        // let j1 = app.run_sysev_loop();

        // let j2 = app.kber.run_loop();

        // join_all(vec![j1, j2]).await;

        Ok(())
    }
}
