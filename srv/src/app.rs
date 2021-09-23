use clap::{AppSettings, Clap};

use futures::future::join_all;
use mytool_core::{app::App as CoreApp, app::Result, opts::AppOpts};

use crate::service::{AgendaService, Service};

/// my tool
#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// config path
    #[clap(short, long, default_value = "/home/yydcnjjw/.my-tool/srv/config.toml")]
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
    pub fn new() -> Result<Self> {
        Ok(Self {
            instance: CoreApp::<Opts>::new()?,
            services: Vec::new(),
        })
    }

    pub async fn run(&mut self) {
        self.services.push(Box::new(
            AgendaService::from_config("/home/yydcnjjw/.my-tool/srv/agenda.toml").unwrap(),
        ));

        join_all(self.services.iter_mut().map(|srv| srv.run())).await;
    }
}
