use std::{collections::HashMap, io, path::Path};

use agenda::Task;
use async_trait::async_trait;
use futures::future::join_all;
use thiserror::Error;

use super::Service;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct AgendaService {
    tasks: HashMap<String, Task>,
}

impl AgendaService {
    pub fn from_config<P: AsRef<Path>>(config: P) -> Result<Self> {
        Ok(Self {
            tasks: AgendaService::parse_config_file(config)?,
        })
    }

    fn parse_config_file<P: AsRef<Path>>(config: P) -> Result<HashMap<String, Task>> {
        Ok(toml::from_str(&std::fs::read_to_string(config)?)?)
    }
}

#[async_trait]
impl Service for AgendaService {
    async fn run(&mut self) {
        join_all(self.tasks.iter_mut().map(|(_, task)| task.run())).await;
    }
}
