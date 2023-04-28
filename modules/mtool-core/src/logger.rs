use std::{env, path::PathBuf, str::FromStr, sync::Arc};

use async_trait::async_trait;
use clap::{arg, ArgMatches};
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, UtcOffset};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, time::OffsetTime},
    EnvFilter,
};

use mapp::{
    define_label,
    provider::{Injector, Res, Take},
    AppContext, AppModule, Tracing,
};

use crate::{Cmdline, CmdlineStage};

use super::ConfigStore;

#[derive(Default)]
pub struct Module {}

define_label!(LoggerStage, Init);

#[derive(Debug, Clone)]
struct Logger {
    _guard: Arc<WorkerGuard>,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    path: Option<PathBuf>,
    name: Option<String>,
    filter: Option<String>,
}

impl Config {
    async fn get_path(&self, cs: &Res<ConfigStore>) -> PathBuf {
        self.path
            .as_ref()
            .unwrap_or(&cs.root_path().await.join("log"))
            .clone()
    }

    fn get_name(&self) -> String {
        self.name
            .as_ref()
            .unwrap_or(&"mtool.log".to_string())
            .clone()
    }
}

#[async_trait]
impl AppModule for Module {
    fn early_init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector()
            .insert(Take::new(OffsetTime::local_rfc_3339().unwrap_or(
                OffsetTime::new(UtcOffset::from_hms(8, 0, 0)?, Rfc3339),
            )));

        Ok(())
    }

    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .insert_stage(CmdlineStage::Init, LoggerStage::Init)
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(LoggerStage::Init, init);
        Ok(())
    }
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline.setup(|cmdline| Ok(cmdline.arg(arg!(--stdout "log output to stdout"))))
}

async fn init(
    injector: Injector,
    cs: Res<ConfigStore>,
    tracing: Res<Tracing>,
    time: Take<OffsetTime<Rfc3339>>,
    args: Res<ArgMatches>,
) -> Result<(), anyhow::Error> {
    if args.get_flag("stdout") {
        info!("Redirecting the logs to the standard output stream.");
        return Ok(());
    }

    let cfg = cs.get::<Config>("logger").await?;

    let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(
        cfg.get_path(&cs).await,
        cfg.get_name(),
    ));

    tracing.set_filter(EnvFilter::from_str(
        &env::var("MTOOL_LOG").unwrap_or(cfg.filter.unwrap_or("info".into())),
    )?)?;
    tracing.set_layer(
        fmt::layer()
            .with_ansi(false)
            .with_timer(time.take()?)
            .with_writer(writer)
            .with_thread_ids(true)
            .with_thread_names(true),
    )?;

    injector.insert(Logger {
        _guard: Arc::new(guard),
    });

    info!("logger is initialized");

    Ok(())
}
