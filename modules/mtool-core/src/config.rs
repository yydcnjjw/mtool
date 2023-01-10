use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use clap::{arg, value_parser, ArgMatches};
use mapp::{AppContext, AppModule, Res};
use tokio::{fs, sync::RwLock};
use toml::macros::Deserialize;

use super::{Cmdline, StartupStage};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct(ConfigStore::new).await;

        app.schedule()
            .add_task(StartupStage::Startup, setup_cmdline)
            .await;

        Ok(())
    }
}

struct ConfigInner {
    root_path: PathBuf,
    table: toml::Value,
}

impl ConfigInner {
    async fn new<T>(path: T) -> Result<Self, anyhow::Error>
    where
        T: Into<PathBuf>,
    {
        let root_path: PathBuf = path.into();

        let s = fs::read_to_string(root_path.join("config.toml"))
            .await
            .context(format!(
                "Failed to load configuration file: {}",
                root_path.display()
            ))?;

        let table = toml::from_str(&s).context("Failed to parse toml file")?;

        Ok(Self { root_path, table })
    }

    fn get<T>(&self, keys: &str) -> Result<T, anyhow::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut value: Option<&toml::Value> = Some(&self.table);

        for key in keys.split(".") {
            value = Some(
                value
                    .unwrap()
                    .get(key)
                    .context(format!("{} is not exist", keys))?,
            );
        }

        value
            .unwrap()
            .clone()
            .try_into()
            .context(format!("Failed to parse {}", keys))
    }

    fn root_path(&self) -> &Path {
        self.root_path.as_path()
    }
}

pub struct ConfigStore {
    inner: RwLock<ConfigInner>,
    daemon: bool,
}

impl ConfigStore {
    async fn new(args: Res<ArgMatches>) -> Result<Res<Self>, anyhow::Error> {
        let config_dir = args.get_one::<PathBuf>("config").unwrap();

        Ok(Res::new(Self {
            inner: RwLock::new(ConfigInner::new(config_dir).await?),
            daemon: args.get_flag("daemon"),
        }))
    }

    pub async fn root_path(&self) -> PathBuf {
        self.inner.read().await.root_path().to_owned()
    }

    pub async fn get<T>(&self, key: &str) -> Result<T, anyhow::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner.read().await.get(key)
    }

    pub fn is_daemon(&self) -> bool {
        self.daemon
    }

    pub fn is_cli(&self) -> bool {
        !self.daemon
    }
}

static mut DEFAULT_CONFIG_DIR: Option<&'static PathBuf> = None;

fn init_default_config_dir() -> Result<(), anyhow::Error> {
    let cfg_dir = dirs::home_dir()
        .map(|p| p.join(".mtool"))
        .context("Failed to get default_config_dir")?;

    unsafe {
        DEFAULT_CONFIG_DIR = Some(Box::leak(Box::new(cfg_dir)));
    }
    Ok(())
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    init_default_config_dir()?;

    cmdline
        .setup(|cmdline| {
            Ok(cmdline
                .arg(
                    arg!(-c --config <FILE> "configuration directory")
                        .value_parser(value_parser!(PathBuf))
                        .default_value(unsafe { DEFAULT_CONFIG_DIR.unwrap().as_os_str() }),
                )
                .arg(arg!(--daemon "daemon mode")))
        })
        .await?;

    Ok(())
}

pub async fn is_daemon(config: Res<ConfigStore>) -> Result<bool, anyhow::Error> {
    Ok(config.is_daemon())
}

pub async fn is_cli(config: Res<ConfigStore>) -> Result<bool, anyhow::Error> {
    Ok(config.is_cli())
}
