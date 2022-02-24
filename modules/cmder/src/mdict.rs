use clap::Clap;
use mdict::common::MdResource;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{app::App, error};
use async_trait::async_trait;

use super::CommandRunner;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clap)]
pub struct MdictCmd {
    /// query
    #[clap(required(true), index(1))]
    query: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MdictConfig {
    pub dict_path: String,
}

impl MdictConfig {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    mdict: MdictConfig,
}

#[async_trait]
impl CommandRunner for MdictCmd {
    async fn run(&self, app: &App) -> error::Result<()> {
        let config: Config = app.config.get(&"dict".to_string())?;

        let mut handles = Vec::new();

        for path in config.mdict.list_dict_paths().await? {
            let query = self.query.clone();
            handles.push(tokio::spawn(async move {
                match mdict::parse(&path) {
                    Ok(mut md) => {
                        let html = md
                            .search(&query)
                            .iter()
                            .filter_map(|item| match &item.1 {
                                MdResource::Text(text) => Some(text),
                                _ => None,
                            })
                            .fold(String::new(), |lhs, rhs| lhs + rhs + "<div></div>");
                        println!("{}{}", md.meta.description, html);
                    }
                    Err(e) => println!("{}", e),
                };
            }));
        }

        for handle in handles {
            handle.await.expect("exec");
        }

        Ok(())
    }
}
