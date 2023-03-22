use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::Filtered,
    fmt,
    layer::{Filter, Layer},
    prelude::*,
    reload, EnvFilter, Registry,
};

type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
type BoxedFilter<S> = Box<dyn Filter<S> + Send + Sync + 'static>;

pub type LoggerLayer<S> = Filtered<BoxedLayer<S>, BoxedFilter<S>, S>;

pub struct Tracing {
    logger: reload::Handle<LoggerLayer<Registry>, Registry>,
    _logger_guard: WorkerGuard,
}

impl Tracing {
    pub fn new() -> Result<Self, anyhow::Error> {
        let (writer, _logger_guard) = tracing_appender::non_blocking(std::io::stdout());

        let filter: BoxedFilter<Registry> = Box::new(EnvFilter::from_env("MTOOL_LOG"));

        let (logger_layer, logger) = reload::Layer::new(
            fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .with_writer(writer)
                .boxed()
                .with_filter(filter),
        );

        tracing_subscriber::registry()
            .with(logger_layer)
            .try_init()
            .context("tracing subscriber init")?;

        Ok(Self {
            logger,
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
