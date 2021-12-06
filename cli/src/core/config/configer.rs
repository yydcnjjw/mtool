use std::{
    fmt::{self, Display},
    path::PathBuf,
};
use tokio::fs;

use anyhow::Context;
use thiserror::Error;
use toml::Value;

use crate::{app::QuitApp, core::evbus::{Event, Receiver, ResponsiveEvent, Sender, post_result}};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct GetConfig {
    key: String,
}

impl GetConfig {
    pub async fn post<'de, T>(sender: &Sender, key: String) -> anyhow::Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        post_result::<GetConfig, anyhow::Result<Value>>(sender, GetConfig { key: key.clone() })
            .await??
            .try_into()
            .with_context(|| format!("Get config value {}", key))
    }
}

#[derive(Debug)]
pub struct Configer {
    table: toml::value::Table,
}

trait PathStr {
    fn str_or_default(&self) -> &str;
}

impl PathStr for PathBuf {
    fn str_or_default(&self) -> &str {
        self.to_str().unwrap_or_default()
    }
}

impl Configer {
    // BUG: https://github.com/rust-lang/rust/issues/50133
    pub async fn load<T>(path: T) -> Result<Self>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();

        let s = fs::read_to_string(path.as_path())
            .await
            .with_context(|| format!("Read config {}", path.str_or_default()))?;
        let table = toml::from_str(&s)
            .with_context(|| format!("Parse config {}", path.str_or_default()))?;

        Ok(Self { table })
    }

    fn get(&self, key: &str) -> Result<Value> {
        Ok(self
            .table
            .get(key)
            .ok_or(Error::KeyNotFound(key.to_string()))?
            .clone())
    }

    pub async fn run_loop(mut rx: Receiver) -> anyhow::Result<()> {
        let cfger = Configer::load(config_path().context("Get config path")?).await?;

        while let Ok(e) = rx.recv().await {
            if let Some(e) = e.downcast_ref::<ResponsiveEvent<GetConfig, anyhow::Result<Value>>>() {
                e.result(
                    cfger
                        .get(&e.key)
                        .context(format!("Get config value {}", e.key)),
                );
            } else if let Some(_) = e.downcast_ref::<Event<QuitApp>>() {
                break;
            }
        }
        Ok(())
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".my-tool").join("config.toml"))
}

impl Display for Configer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = toml::to_string_pretty(&self.table).context("Serialize config") {
            write!(f, "{}", s)
        } else {
            Err(fmt::Error)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    static DEFAULT_MOCK_CONFIG: &str = r#"
[module1]
value = "*"

[module2]
value = "*"
"#;

    fn prepare_config_file(s: &str) -> anyhow::Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", s)?;
        Ok(file)
    }

    async fn new_config(s: &str) -> anyhow::Result<Configer> {
        let file = prepare_config_file(s).unwrap();
        Ok(Configer::load(file.path()).await?)
    }

    #[tokio::test]
    async fn test_config_try_from() {
        let file = prepare_config_file(DEFAULT_MOCK_CONFIG).unwrap();
        Configer::load(file.path()).await.unwrap();
        Configer::load(file.path().to_str().unwrap()).await.unwrap();
    }

    #[tokio::test]
    async fn test_config_get() {
        let config = new_config(DEFAULT_MOCK_CONFIG).await.unwrap();
        let module1 = config.get("module1").unwrap();
        assert_eq!(module1["value"].as_str(), Some("*"));
        let module2 = config.get("module2").unwrap();
        assert_eq!(module2["value"].as_str(), Some("*"));
    }
}
