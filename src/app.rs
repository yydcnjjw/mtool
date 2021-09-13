use anyhow::Context;
use clap::Clap;
use log::info;

pub use crate::error::Result;
use crate::{config::Config, opts::Opts};

pub struct App {
    pub opts: Opts,
    pub config: Config,
}

impl App {
    pub fn new() -> Result<App> {
        pretty_env_logger::init();

        let opts = Opts::parse();

        let config = Config::load(&opts.config_path)
            .with_context(|| format!("Load config {}", opts.config_path))?;

        Ok(App { opts, config })
    }

    pub async fn run(&self) -> Result<()> {
        self.opts.subcmd.exec(self).await
    }
}
