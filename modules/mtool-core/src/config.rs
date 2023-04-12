use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use clap::{arg, value_parser, ArgMatches};
use futures::{future::BoxFuture, FutureExt};
use mapp::{provider::Res, AppContext, AppModule};
use tokio::{fs, sync::RwLock};
use toml::macros::Deserialize;

use crate::CmdlineStage;

use super::Cmdline;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(ConfigStore::new);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline);

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
        let mut value: &toml::Value = &self.table;

        for key in keys.split(".") {
            value = value.get(key).context(format!("{} is not exist", keys))?;
        }

        value
            .clone()
            .try_into()
            .context(format!("Failed to parse {}", keys))
    }

    fn root_path(&self) -> &Path {
        self.root_path.as_path()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum StartupMode {
    WGui,
    Tui,
    Cli,
}

impl From<&str> for StartupMode {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "wgui" => StartupMode::WGui,
            "tui" => StartupMode::Tui,
            "cli" => StartupMode::Cli,
            _ => unreachable!(),
        }
    }
}

pub struct ConfigStore {
    inner: RwLock<ConfigInner>,
    mode: StartupMode,
}

impl ConfigStore {
    async fn new(args: Res<ArgMatches>) -> Result<Res<Self>, anyhow::Error> {
        let config_dir = args.get_one::<PathBuf>("config").unwrap();

        Ok(Res::new(Self {
            inner: RwLock::new(ConfigInner::new(config_dir).await?),
            mode: StartupMode::from(args.get_one::<String>("mode").unwrap().as_str()),
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

    pub fn startup_mode(&self) -> StartupMode {
        self.mode
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

    cmdline.setup(|cmdline| {
        Ok(cmdline
            .arg(
                arg!(-c --config <FILE> "configuration directory")
                    .value_parser(value_parser!(PathBuf))
                    .default_value(unsafe { DEFAULT_CONFIG_DIR.unwrap().as_os_str() }),
            )
            // .arg(arg!(--daemon "daemon mode")))
            .arg(
                arg!(--mode <MODE> "startup mode")
                    .value_parser(["cli", "wgui", "tui"])
                    .default_value("cli"),
            ))
    })?;

    Ok(())
}

pub fn is_startup_mode(
    mode: StartupMode,
) -> impl Fn(Res<ConfigStore>) -> BoxFuture<'static, Result<bool, anyhow::Error>> + Clone {
    move |config: Res<ConfigStore>| async move { Ok(config.startup_mode() == mode) }.boxed()
}

pub fn not_startup_mode(
    mode: StartupMode,
) -> impl Fn(Res<ConfigStore>) -> BoxFuture<'static, Result<bool, anyhow::Error>> + Clone {
    move |config: Res<ConfigStore>| async move { Ok(config.startup_mode() != mode) }.boxed()
}
