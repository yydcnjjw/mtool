use std::{fs, path::PathBuf};

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

impl Config {
    fn path_str(&self) -> &str {
        self.path.to_str().unwrap_or_default()
    }
}

impl Config {
    pub fn load<T>(path: &T) -> Result<Config>
    where
        T: Into<PathBuf> + Clone,
    {
        let path = PathBuf::from(path.clone().into());
        let path_str = path.to_str().unwrap_or_default();
        let table = toml::from_str(
            &fs::read_to_string(path.as_path())
                .with_context(|| format!("read config {}", path_str))?,
        )
        .with_context(|| format!("parse config {}", path_str))?;
        Ok(Config { path, table })
    }

    pub fn store(&self) -> Result<()> {
        Ok(fs::write(
            self.path.as_path(),
            &toml::to_string_pretty(&self.table)
                .with_context(|| format!("serialize config {}", self.path_str()))?,
        )
        .with_context(|| format!("write config {}", self.path_str()))?)
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
