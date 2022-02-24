use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use clap::Parser;
use cmder_mod::Command;
use mdict::common::MdResource;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MdictConfig {
    path: String,
}

impl MdictConfig {
    async fn list_dict_paths(&self) -> anyhow::Result<Vec<PathBuf>> {
        Ok(fs::read_dir(&self.path)?
            .filter_map(|path| path.ok())
            .filter_map(|path| {
                if let Ok(t) = path.file_type() {
                    if t.is_dir() {
                        None
                    } else {
                        Some(path.path())
                    }
                } else {
                    None
                }
            })
            .collect())
    }
}

#[derive(Debug)]
pub struct Cmd {
    cfg: MdictConfig,
}

impl Cmd {
    pub fn new(cfg: MdictConfig) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { cfg }))
    }
}

/// Dict module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// query
    query: String,
}

impl Cmd {
    async fn execute(&mut self, args: &Vec<String>) -> anyhow::Result<()> {
        let args = Args::try_parse_from(args).context("Failed to parse dict args")?;

        let mut handles = Vec::new();
        for path in self.cfg.list_dict_paths().await? {
            let query = args.query.clone();
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
                    Err(e) => println!("{:?}", e),
                };
            }));
        }

        for handle in handles {
            handle.await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, args: Vec<String>) {
        if let Err(e) = self.execute(&args).await {
            log::warn!("{:?}", e);
        }
    }
}
