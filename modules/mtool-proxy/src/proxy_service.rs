use mapp::{
    anyhow::{self, Context},
    itertools::Itertools,
    prelude::*,
    serde::Deserialize,
    sync::Mutex,
    tokio::fs,
    toml,
};
use mproxy::{protos::geosite, router::GeositeFile, App, AppConfig};
use mtool_core::ConfigStore;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "mapp::serde")]
struct Config {
    path: PathBuf,
    routing_rule_id: String,
    resource_path: PathBuf,
}

pub struct ProxyService {
    routing_rule_id: String,
    resource: Mutex<GeositeFile>,
    inner: App,
}

impl ProxyService {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<ProxyService>, anyhow::Error> {
        let config = cs.get::<Config>("proxy").context("Failed to parse proxy")?;

        let mut app_config = toml::from_str::<AppConfig>(&fs::read_to_string(config.path).await?)?;

        app_config
            .routing
            .resource
            .push(config.resource_path.clone());

        let app = App::new(app_config)
            .await
            .context("Failed to create proxy service")?;

        Ok(Res::new(Self {
            inner: app,
            routing_rule_id: config.routing_rule_id,
            resource: Mutex::new(GeositeFile::new(&config.resource_path)?),
        }))
    }

    pub async fn add_routing_rule(&self, target: &str) -> Result<(), anyhow::Error> {
        {
            let mut gs = self.resource.lock();
            gs.insert_target("pri", target)?;
            gs.store()?;
        }

        Ok(self
            .inner
            .router()
            .add_rule_target(&self.routing_rule_id, target)?)
    }

    pub async fn remove_routing_rule(&self, target: &geosite::Domain) -> Result<(), anyhow::Error> {
        let mut gs = self.resource.lock();
        gs.remove_with_domain("pri", target)?;
        gs.store()?;
        Ok(())
    }

    pub fn routing_rules(&self) -> Vec<geosite::Domain> {
        let gs = self.resource.lock();
        gs.get_site_group("pri")
            .map(|sg| sg.domain.iter().cloned().collect_vec())
            .unwrap_or_default()
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        self.inner.run().await
    }
}
