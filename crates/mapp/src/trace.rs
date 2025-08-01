use anyhow::Context;
use tracing::level_filters::LevelFilter;
#[cfg(not(target_arch = "wasm32"))]
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::Filtered,
    fmt::{self, layer},
    layer::{Filter, Layer},
    prelude::*,
    reload, Registry,
};

type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
type BoxedFilter<S> = Box<dyn Filter<S> + Send + Sync + 'static>;

pub type LoggerLayer<S> = Filtered<BoxedLayer<S>, BoxedFilter<S>, S>;

pub struct Tracing {
    logger: reload::Handle<LoggerLayer<Registry>, Registry>,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    _logger_guard: WorkerGuard,
}

impl Tracing {
    pub fn new() -> Result<Self, anyhow::Error> {
        #[allow(unused)]
        let mut layer = fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true);

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let (layer, _logger_guard, filter) = {
            use time::{format_description::well_known::Rfc3339, UtcOffset};
            use tracing_subscriber::fmt::time::OffsetTime;

            let (writer, logger_guard) = tracing_appender::non_blocking(std::io::stdout());
            (
                layer
                    .with_timer(
                        OffsetTime::local_rfc_3339()
                            .unwrap_or(OffsetTime::new(UtcOffset::from_hms(8, 0, 0)?, Rfc3339)),
                    )
                    .with_ansi(true)
                    .pretty()
                    .with_writer(writer),
                logger_guard,
                Box::new(tracing_subscriber::EnvFilter::from_env("MTOOL_LOG"))
                    as BoxedFilter<Registry>,
            )
        };

        #[cfg(target_os = "android")]
        let (layer, filter) = {
            (
                layer
                    .with_level(false)
                    .with_ansi(false)
                    .without_time()
                    .with_writer(crate::android::LogcatMakeWriter),
                Box::new(LevelFilter::DEBUG) as BoxedFilter<Registry>,
            )
        };

        #[cfg(target_family = "wasm")]
        let (layer, filter) = {
            use tracing::metadata::LevelFilter;
            (
                layer
                    .with_ansi(false)
                    .without_time()
                    .with_writer(tracing_web::MakeConsoleWriter),
                Box::new(LevelFilter::DEBUG) as BoxedFilter<Registry>,
            )
        };

        let layer = { layer.boxed() }.with_filter(filter);

        let (logger_layer, logger) = reload::Layer::new(layer);

        tracing_subscriber::registry()
            .with(logger_layer)
            .try_init()
            .context("tracing subscriber init")?;

        Ok(Self {
            logger,
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            _logger_guard,
        })
    }

    pub fn set_filter<F>(&self, filter: F) -> Result<(), anyhow::Error>
    where
        F: Filter<Registry> + Send + Sync + 'static,
    {
        self.logger
            .modify(|l| *l.filter_mut() = Box::new(filter))
            .context("set_filter")
    }

    pub fn set_layer<L>(&self, layer: L) -> Result<(), anyhow::Error>
    where
        L: Layer<Registry> + Send + Sync + 'static,
    {
        self.logger
            .modify(|l| *l.inner_mut() = Box::new(layer))
            .context("set_layer")?;

        Ok(())
    }
}
