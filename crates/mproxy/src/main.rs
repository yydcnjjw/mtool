use std::path::PathBuf;

use clap::Parser;

use mproxy::{App, AppConfig};
use tokio::fs;
use tracing::debug;

use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let registry = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env());

    #[cfg(feature = "telemetry")]
    let registry = {
        use mproxy::metrics::prometheus_server;
        use opentelemetry::{
            global,
            sdk::{
                export::metrics::aggregation,
                metrics::{controllers, processors, selectors},
                Resource,
            },
            KeyValue,
        };
        use tracing_opentelemetry::MetricsLayer;

        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("mproxy")
            .install_batch(opentelemetry::runtime::Tokio)?;

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .with_resource(Resource::new(vec![KeyValue::new("service.name", "mproxy")]))
        .build();

        tokio::spawn(prometheus_server(controller.clone()));

        registry.with(telemetry).with(MetricsLayer::new(controller))
    };

    registry.try_init()?;

    let args = Args::parse();

    let buf = fs::read(args.config).await?;

    let config = toml::from_slice::<AppConfig>(&buf)?;

    debug!("{:?}", config);

    let app = App::new(config).await?;

    app.run().await?;

    #[cfg(feature = "telemetry")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
