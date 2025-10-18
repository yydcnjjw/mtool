use std::{env, path::PathBuf, str::FromStr, sync::Arc};

use mapp::{
    anyhow,
    cfg_if::cfg_if,
    serde::Deserialize,
    tracing::info,
    tracing_appender::{self, non_blocking::WorkerGuard},
    tracing_subscriber::{
        fmt::{self, time::OffsetTime},
        EnvFilter,
    },
};
use time::{format_description::well_known::Rfc3339, UtcOffset};

use mapp::{define_label, prelude::*};

cfg_if! {
    if #[cfg(feature = "cmdline")] {
        use clap::{arg, ArgMatches};
        use crate::{Cmdline, CmdlineStage};
    }
}

use crate::AppStage;

use super::ConfigStore;

#[derive(Default)]
pub struct Module {}

define_label!(LoggerStage, Init);

#[derive(Debug, Clone)]
struct Logger {
    _guard: Arc<WorkerGuard>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
struct Config {
    path: Option<PathBuf>,
    name: Option<String>,
    filter: Option<String>,
}

impl Config {
    async fn get_path(&self, cs: &Res<ConfigStore>) -> PathBuf {
        self.path
            .as_ref()
            .unwrap_or(&cs.root_path().join("log"))
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
            .insert(Res::new(OffsetTime::local_rfc_3339().unwrap_or(
                OffsetTime::new(UtcOffset::from_hms(8, 0, 0)?, Rfc3339),
            )));

        Ok(())
    }

    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        cfg_if! {
            if #[cfg(feature = "cmdline")] {
                app.schedule()
                .insert_stage(CmdlineStage::Parse, LoggerStage::Init)
                .add_once_task(CmdlineStage::Setup, setup_cmdline);
            } else {
                app.schedule()
                .insert_stage(AppStage::Startup, LoggerStage::Init);
            }
        }

        app.schedule().add_once_task(LoggerStage::Init, init);
        Ok(())
    }
}

#[cfg(feature = "cmdline")]
async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline.setup(|cmdline| {
        Ok(
            cmdline.arg(arg!(--stdout "log output to stdout").default_value(
                #[cfg(debug_assertions)]
                "true",
                #[cfg(not(debug_assertions))]
                "false",
            )),
        )
    })
}

async fn init(
    injector: Injector,
    cs: Res<ConfigStore>,
    tracing: Res<Tracing>,
    time: Take<Res<OffsetTime<Rfc3339>>>,
    args: Option<Res<clap::ArgMatches>>,
) -> Result<(), anyhow::Error> {
    if let Some(args) = args {
        if args.get_flag("stdout") {
            info!("Redirecting the logs to the standard output stream.");
            return Ok(());
        }
    }

    let cfg = cs.get::<Config>("logger")?;

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
