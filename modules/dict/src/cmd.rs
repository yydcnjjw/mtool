use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use clap::Parser;
use cmder_mod::Command;
use mdict::decode::mdx;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MdictConfig {
    path: String,
}

impl MdictConfig {
    async fn list_dict_paths(&self) -> anyhow::Result<Vec<PathBuf>> {
        let meta = fs::metadata(&self.path).await?;
        let mut vec = Vec::new();
        if meta.is_dir() {
            let mut s = ReadDirStream::new(fs::read_dir(&self.path).await?);
            while let Some(item) = s.next().await {
                if let Ok(item) = item {
                    if let Ok(t) = item.file_type().await {
                        if t.is_file() {
                            vec.push(item.path());
                        }
                    }
                }
            }
        } else {
            vec.push(PathBuf::from_str(&self.path).context("Failed to parse dict path")?);
        }
        Ok(vec)
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
    async fn query<P>(q: String, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let data = fs::read(path).await?;

        let mut md = mdx::parse(data.as_slice()).context("Failed to parse mdx")?;
        let html = md
            .search(&q)
            .iter()
            .filter_map(|item| match &item.1 {
                mdx::Resource::Text(text) => Some(text),
                _ => None,
            })
            .fold(String::new(), |lhs, rhs| lhs + rhs + "<div></div>");
        println!("{}{}", md.meta.description, html);

        Ok(())
    }

    async fn execute(&mut self, args: &Vec<String>) -> anyhow::Result<()> {
        let args = Args::try_parse_from(args).context("Failed to parse dict args")?;

        let mut handles = Vec::new();
        for path in self.cfg.list_dict_paths().await? {
            handles.push(tokio::spawn(Self::query(args.query.clone(), path)));
        }

        for handle in handles {
            handle.await??;
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
