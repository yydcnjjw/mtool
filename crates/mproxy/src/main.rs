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
    let (registry, _) = {
        use mproxy::metrics::new_metrics_layer;
        let (layer, drop) = new_metrics_layer()?;
        (registry.with(layer), drop)
    };

    registry.try_init()?;

    let args = Args::parse();

    let buf = fs::read_to_string(args.config).await?;

    let config = toml::from_str::<AppConfig>(&buf)?;

    debug!("{:?}", config);

    let app = App::new(config).await?;

    app.run().await?;

    Ok(())
}
