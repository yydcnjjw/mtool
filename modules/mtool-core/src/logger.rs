use std::{env, path::PathBuf, str::FromStr};

use async_trait::async_trait;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Deserializers, Root},
    Handle,
};
use mapp::{define_label, AppContext, AppModule, CreateTaskDescriptor, Label, Res};

use super::{Cmdline, ConfigStore, InitStage, StartupStage};

#[derive(Default)]
pub struct Module {}

define_label!(LogStage, Init);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_task(StartupStage::Startup, setup_cmdline)
            .await
            .add_task(InitStage::PreInit, init.label(LogStage::Init))
            .await;
        Ok(())
    }
}

static mut LOGGER_HANDLE: Option<&'static Handle> = None;

pub fn early_init() {
    let stdout = ConsoleAppender::builder().build();

    let level = log::LevelFilter::from_str(
        &env::var("RUST_LOG")
            .unwrap_or(String::from("INFO"))
            .to_lowercase(),
    )
    .unwrap();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
    unsafe {
        LOGGER_HANDLE = Some(Box::leak(Box::new(handle)));
    }
}

async fn setup_cmdline(_cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    // cmdline
    //     .setup(|cmdline| Ok(cmdline.arg(arg!(-d --debug ... "Turn debugging information on"))))
    //     .await?;

    Ok(())
}

async fn logger_config_file(config: &Res<ConfigStore>) -> PathBuf {
    config.root_path().await.join("log4rs.yaml")
}

async fn init(config: Res<ConfigStore>) -> Result<(), anyhow::Error> {
    let log_cfg_file = logger_config_file(&config).await;

    let cfg = log4rs::config::load_config_file(&log_cfg_file, Deserializers::default())?;

    let handle = unsafe { LOGGER_HANDLE.unwrap() };
    handle.set_config(cfg);

    // match args.get_count("debug") {
    //     1 => log::set_max_level(log::LevelFilter::Debug),
    //     2 => log::set_max_level(log::LevelFilter::Trace),
    //     0 | _ => {}
    // }

    log::debug!("init log4rs from {}", log_cfg_file.display());

    Ok(())
}
