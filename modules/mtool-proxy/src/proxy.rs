use std::{path::PathBuf, sync::Mutex};

use anyhow::Context;
use mapp::provider::Res;
use mproxy::{router::GeositeFile, App, AppConfig};
use mtool_core::ConfigStore;
use serde::Deserialize;
use tokio::fs;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    path: PathBuf,
    proxy_id: String,
    resource_path: PathBuf,
}

pub struct ProxyApp {
    pub proxy_id: String,
    pub resource: Mutex<GeositeFile>,
    pub inner: App,
}

impl ProxyApp {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<ProxyApp>, anyhow::Error> {
        let config = cs
            .get::<Config>("proxy")
            .await
            .context("Failed to parse proxy")?;

        let mut app_config = toml::from_str::<AppConfig>(&fs::read_to_string(config.path).await?)?;

        app_config
            .routing
            .resource
            .push(config.resource_path.clone());

        let app = App::new(app_config)
            .await
            .context("Failed to create proxy")?;

        Ok(Res::new(Self {
            inner: app,
            proxy_id: config.proxy_id,
            resource: Mutex::new(GeositeFile::new(&config.resource_path)?),
        }))
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        self.inner.run().await
    }
}
