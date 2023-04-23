use std::{env, path::PathBuf, str::FromStr, sync::Arc};

use async_trait::async_trait;
use serde::Deserialize;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

use mapp::{
    define_label,
    provider::{Injector, Res},
    AppContext, AppModule, Tracing,
};

use crate::CmdlineStage;

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
    filter: String,
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
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .insert_stage(CmdlineStage::Init, LoggerStage::Init)
            .add_once_task(LoggerStage::Init, init);
        Ok(())
    }
}

async fn init(
    injector: Injector,
    cs: Res<ConfigStore>,
    tracing: Res<Tracing>,
) -> Result<(), anyhow::Error> {
    let cfg = cs.get::<Config>("logger").await?;

    let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(
        cfg.get_path(&cs).await,
        cfg.get_name(),
    ));

    tracing.set_filter(EnvFilter::from_str(
        &env::var("MTOOL_LOG").unwrap_or(cfg.filter),
    )?)?;
    tracing.set_layer(
        fmt::layer()
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
