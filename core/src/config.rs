use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
};
use tokio::fs;

use anyhow::Context;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("key not found: {0}")]
    KeyNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Config {
    name: String,
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

impl Config {
    // BUG: https://github.com/rust-lang/rust/issues/50133
    pub async fn load<T>(path: T) -> Result<Config>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();

        let name = path
            .file_name()
            .with_context(|| format!("Config must be file: {}", path.str_or_default()))?;

        let s = fs::read_to_string(path.as_path())
            .await
            .with_context(|| format!("Read config {}", path.str_or_default()))?;
        let table = toml::from_str(&s)
            .with_context(|| format!("Parse config {}", path.str_or_default()))?;

        Ok(Config { name, table })
    }

    pub async fn store(&self) -> Result<()> {
        Ok(fs::write(self.path.as_path(), &self.serialize_config()?)
            .await
            .with_context(|| format!("Write config {}", self.name))?)
    }

    fn serialize_config(&self) -> Result<String> {
        Ok(toml::to_string_pretty(&self.table)
            .with_context(|| format!("Serialize config {}", self.name))?)
    }

    pub async fn insert<T>(&mut self, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.table.insert(
            key.to_string(),
            toml::Value::try_from(value).with_context(|| format!("Store key {}", key))?,
        );

        Ok(())
    }

    pub async fn insert_store<T>(&mut self, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.insert(key, value).await?;
        self.store().await?;
        Ok(())
    }

    pub async fn get<'de, T>(&self, key: &str) -> Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        Ok(self
            .table
            .get(key)
            .ok_or(Error::KeyNotFound(key.to_string()))?
            .clone()
            .try_into()
            .with_context(|| format!("Get config value {}", key))?)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "path: {}\n{}",
            self.path().to_str().unwrap_or_default(),
            self.serialize_config().unwrap_or_default()
        )
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

    async fn new_config(s: &str) -> anyhow::Result<Config> {
        let file = prepare_config_file(s).unwrap();
        Ok(Config::load(file.path()).await?)
    }

    #[tokio::test]
    async fn test_config_try_from() {
        let file = prepare_config_file(DEFAULT_MOCK_CONFIG).unwrap();
        Config::load(file.path()).await.unwrap();
        Config::load(file.path().to_str().unwrap()).await.unwrap();
    }

    #[tokio::test]
    async fn test_config_get() {
        let config = new_config(DEFAULT_MOCK_CONFIG).await.unwrap();
        let module1 = config.get::<toml::Value>("module1").await.unwrap();
        assert_eq!(module1["value"].as_str(), Some("*"));
        let module2 = config.get::<toml::Value>("module2").await.unwrap();
        assert_eq!(module2["value"].as_str(), Some("*"));
    }

    #[tokio::test]
    async fn test_config_insert() {
        let mut config = new_config(DEFAULT_MOCK_CONFIG).await.unwrap();

        config.insert("module1", "test").await.unwrap();

        assert_eq!(config.get::<String>("module1").await.unwrap(), "test");
    }

    #[tokio::test]
    async fn test_config_insert_store() {
        let mut config = new_config(DEFAULT_MOCK_CONFIG).await.unwrap();

        config.insert_store("module1", "test").await.unwrap();

        assert_eq!(config.get::<String>("module1").await.unwrap(), "test");
    }
}
