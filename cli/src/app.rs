use std::{env, path::PathBuf};

use anyhow::Context;
use mytool_core::config::Config;

use crate::command::Commander;

// use mytool_core::app::{App, Result};

// use super::opts::Opts;

// pub async fn run() -> Result<()> {
//     let app = App::<Opts>::new().await?;
//     app.opts.subcmd.exec(&app).await
// }

pub struct App {
    pub cfg: Config,
    pub cmder: Commander,
}

fn config_path() -> Option<PathBuf> {
    env::home_dir().map(|p| p.join(".my-tool").join("config.toml"))
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            cfg: Config::load(config_path().context("Get config path")?).await?,
            cmder: Commander::new(),
        })
    }
}
