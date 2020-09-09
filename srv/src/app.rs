use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;

use futures::future::join_all;

use crate::service::{AgendaService, Service};
use mytool_core::{
    app::{App as CoreApp, Result as AppResult},
    config::{self, Config as AppConfig},
};

pub struct App {
    instance: CoreApp,
    services: Vec<Box<dyn Service>>,
}

struct Config {}

impl Config {
    async fn load(appconfig: &AppConfig) -> config::Result<Config> {
        let config = appconfig
            .get::<toml::Value>("server")
            .await
            .with_context("Server read Value server")?;
        if let path = Some(config.get("include")) {
            AppConfig::load(path)
        } 
    }
}

impl App {
    pub async fn new() -> AppResult<Self> {
        Ok(Self {
            instance: CoreApp::new().await?,
            services: Vec::new(),
        })
    }

    async fn get_config(&mut self) -> &AppConfig {
        let cfg = self
            .instance
            .get_config()
            .get::<toml::Value>("server")
            .await
            .unwrap();
        cfg.get("include")
    }

    pub async fn run(&mut self) {
        {
            let srv = Box::new(AgendaService::new(self).unwrap());
            self.services.push(srv);
        }

        join_all(self.services.iter_mut().map(|srv| srv.run())).await;
    }
}
