use std::path::PathBuf;

use anyhow::Context;
use clap::{AppSettings, Clap};

use futures::future::join_all;

use crate::service::{AgendaService, Service};
use mytool_core::{
    app::{App as CoreApp, Result as AppResult},
    opts::AppOpts,
};

static APP_NAME: &str = "my-tool";

fn default_config_dir() -> String {
    format!(".{}/server", APP_NAME)
}

fn default_config_path() -> String {
    dirs::home_dir()
        .context("Home dir is not found")
        .unwrap_or(PathBuf::new())
        .join(default_config_dir())
        .to_str()
        .unwrap_or_default()
        .to_string()
}

/// my tool
#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// config path
    #[clap(short, long, default_value = "default_config_path")]
    pub config_path: String,
}

impl AppOpts for Opts {
    fn config_path(&self) -> &str {
        self.config_path.as_str()
    }
}

pub struct App {
    instance: CoreApp<Opts>,
    services: Vec<Box<dyn Service>>,
}

impl App {
    pub async fn new() -> AppResult<Self> {
        Ok(Self {
            instance: CoreApp::<Opts>::new().await?,
            services: Vec::new(),
        })
    }

    pub async fn run(&mut self) {
        {
            let srv = Box::new(AgendaService::new(self).unwrap());
            self.services.push(srv);
        }

        join_all(self.services.iter_mut().map(|srv| srv.run())).await;
    }
}
