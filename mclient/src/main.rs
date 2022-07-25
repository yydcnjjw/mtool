mod app;
mod daemon;
mod path;

use anyhow::Context;
use app::App;
use clap::Parser;
use daemon::daemon;

/// mtool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// daemon
    #[clap(long, value_parser, default_value_t = false)]
    daemon: bool,
}

fn logger_setup() -> anyhow::Result<()> {
    log4rs::init_file(
        path::logger_config_file().context("Failed to get logger config file")?,
        Default::default(),
    )?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = logger_setup() {
        println!("{:?}", e);
        return;
    }

    if args.daemon {
        if let Err(e) = daemon() {
            log::error!("{:?}", e);
            return;
        }
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            if let Err(e) = App::run(args).await {
                log::error!("{:?}", e);
            }
        });
}
