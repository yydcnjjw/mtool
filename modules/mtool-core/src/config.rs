use std::path::{Path, PathBuf};

use mapp::{
    anyhow::{self, anyhow, Context},
    prelude::*,
    sync::RwLock,
    tokio::fs,
    toml::{self, macros::Deserialize},
};

use crate::CmdlineStage;

use super::Cmdline;

pub(crate) struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            use clap::{arg, value_parser, ArgMatches};

            async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
                let config_dir = dirs::home_dir()
                    .map(|p| p.join(".mtool"))
                    .context("Failed to get config_dir")?;

                cmdline.setup(move |cmdline| {
                    Ok(cmdline
                        .arg(
                            arg!(-c --config <FILE> "configuration directory")
                                .value_parser(value_parser!(PathBuf))
                                .default_value_os(config_dir.into_os_string()),
                        )
                        .arg(
                            arg!(--mode <MODE> "startup mode")
                                .value_parser(["cli", "wgui", "tui"])
                                .default_value("cli"),
                        ))
                })?;

                Ok(())
            }

            app.schedule()
                .add_once_task(CmdlineStage::Setup, setup_cmdline);

            async fn config_store(
                args: Res<ArgMatches>,
            ) -> Result<Res<ConfigStore>, anyhow::Error> {
                let config_dir = args
                    .get_one::<PathBuf>("config")
                    .ok_or(anyhow!("missing config"))?;

                Ok(Res::new(ConfigStore {
                    inner: RwLock::new(ConfigInner::new(config_dir).await?),
                }))
            }
            app.injector().construct_once(config_store);
        }

        #[cfg(target_os = "android")]
        {
            use mapp::android_activity::AndroidApp;
            async fn config_store(
                android_app: Res<AndroidApp>,
            ) -> Result<Res<ConfigStore>, anyhow::Error> {
                let config_dir = android_app
                    .external_data_path()
                    .context("missing external data path")?
                    .join("config");

                Ok(Res::new(ConfigStore {
                    inner: RwLock::new(ConfigInner::new(config_dir).await?),
                }))
            }
            app.injector().construct_once(config_store);
        }

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

    fn get_optional<T>(&self, keys: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut value: &toml::Value = &self.table;

        for key in keys.split(".") {
            if let Some(child) = value.get(key) {
                value = child;
            }
        }

        value.clone().try_into().ok()
    }

    fn get<T>(&self, keys: &str) -> Result<T, anyhow::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut value: &toml::Value = &self.table;

        for key in keys.split(".") {
            value = value
                .get(key)
                .context(format!("{} field is not exist", keys))?;
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

pub struct ConfigStore {
    inner: RwLock<ConfigInner>,
}

impl ConfigStore {
    pub async fn root_path(&self) -> PathBuf {
        self.inner.read().root_path().to_owned()
    }

    pub async fn get<T>(&self, key: &str) -> Result<T, anyhow::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner.read().get(key)
    }

    pub fn get_optional<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner.read().get_optional(key)
    }
}
