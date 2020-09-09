use std::path::{Path, PathBuf};

use anyhow::Context;

use super::config::Config;
use clap::{AppSettings, Clap};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    App(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct AppOpts {
    /// config path
    #[clap(short, long, default_value = ".my-tool")]
    config: String,
}

impl AppOpts {
    pub fn app_config(&self) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.config);
        p.push("app.toml");
        p
    }
}

pub struct App {
    opts: AppOpts,
    config: Config,
}

impl App {
    pub async fn new() -> Result<Self> {
        pretty_env_logger::init_timed();

        let opts = AppOpts::parse();

        let config = Config::load(&opts.app_config())
            .await
            .with_context(|| format!("Load config {}", opts.config))?;

        Ok(Self { opts, config })
    }

    pub fn get_config(&mut self) -> &Config {
        &self.config
    }
}
