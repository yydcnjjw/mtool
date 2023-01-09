use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use clap::Parser;
use mapp::Res;
use mdict::decode::mdx;
use mtool_cmder::CommandArgs;
use serde::Deserialize;
use tokio::fs;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::DictConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct MdictConfig {
    path: String,
}

impl MdictConfig {
    async fn list_dict_paths(&self) -> Result<Vec<PathBuf>, anyhow::Error> {
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

/// Dict module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    /// query
    query: String,
}

async fn query<P>(q: String, path: P) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
{
    let data = fs::read(path).await?;

    let md = mdx::parse(data.as_slice()).context("Failed to parse mdx")?;

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

pub async fn mdx(args: Res<CommandArgs>, cfg: Res<DictConfig>) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            return Ok(());
        }
    };

    let mut handles = Vec::new();
    for path in cfg.mdict.list_dict_paths().await? {
        handles.push(tokio::spawn(query(args.query.clone(), path)));
    }

    for handle in handles {
        handle.await??;
    }
    Ok(())
}
