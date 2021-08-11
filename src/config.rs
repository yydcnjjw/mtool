use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;
use serde::de::Deserialize;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    De(#[from] toml::de::Error),
    #[error("{0}")]
    Ser(#[from] toml::ser::Error),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("KeyNotFound {0}")]
    KeyNotFound(String),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Config {
    path: PathBuf,
    table: toml::value::Table,
}

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        Ok(Config {
            path: path.to_path_buf(),
            table: toml::from_str(&fs::read_to_string(path)?)?,
        })
    }

    pub fn store(&self) -> Result<()> {
        Ok(fs::write(
            self.path.as_path(),
            &toml::to_string_pretty(&self.table)?,
        )?)
    }

    pub fn get<'de, T>(&self, key: &String) -> Result<T>
    where T: Deserialize<'de> {
        Ok(self.table.get(key).ok_or(Error::KeyNotFound(key.clone()))?.try_into()?)
    }
}
