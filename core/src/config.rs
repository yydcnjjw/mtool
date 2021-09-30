use std::path::{Path, PathBuf};
use std::fs;

use anyhow::Context;
use serde::de::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("key not found: {0}")]
    KeyNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
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
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    // BUG: https://github.com/rust-lang/rust/issues/50133
    pub fn try_from<T>(path: T) -> Result<Config>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        let s = fs::read_to_string(path.as_path())
            .with_context(|| format!("Read config {}", path.str_or_default()))?;
        let table = toml::from_str(&s)
            .with_context(|| format!("Parse config {}", path.str_or_default()))?;
        Ok(Config { path, table })
    }

    pub fn store(&self) -> Result<()> {
        Ok(fs::write(
            self.path.as_path(),
            &toml::to_string_pretty(&self.table)
                .with_context(|| format!("serialize config {}", self.path.str_or_default()))?,
        )
        .with_context(|| format!("write config {}", self.path.str_or_default()))?)
    }

    pub fn get<'de, T>(&self, key: &String) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        Ok(self
            .table
            .get(key)
            .ok_or(Error::KeyNotFound(key.clone()))?
            .clone()
            .try_into()
            .with_context(|| format!("get config value {}", key))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_try_from() {
        Config::try_from("test.toml");
    }
}
