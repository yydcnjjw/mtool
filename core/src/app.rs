use anyhow::Context;

use super::{config::Config, opts::AppOpts};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    App(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct App<Opts>
where
    Opts: AppOpts,
{
    pub opts: Opts,
    pub config: Config,
}

impl<Opts> App<Opts>
where
    Opts: AppOpts,
{
    pub fn new() -> Result<Self> {
        pretty_env_logger::init();

        let opts = Opts::parse();

        let config = Config::load(&opts.config_path())
            .with_context(|| format!("Load config {}", opts.config_path()))?;

        Ok(Self { opts, config })
    }

    pub async fn run(&self) -> Result<()> {
        Ok(self.opts.exec_cmd().context("Execute command failed")?)
    }
}
