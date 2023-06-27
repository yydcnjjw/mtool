use std::path::{Path, PathBuf};

use anyhow::Context;
use futures::{
    future::{join_all, try_join_all},
    StreamExt,
};
use mapp::provider::Res;
use mdict::decode::mdx;
use mtool_core::ConfigStore;
use ouroboros::self_referencing;
use serde::Deserialize;
use tokio::{fs, sync::Mutex};
use tokio_stream::wrappers::ReadDirStream;

use super::QueryResult;

#[derive(Debug, Deserialize, Clone)]
struct Config {
    path: String,
}

type DictStorage = Vec<u8>;

#[self_referencing]
struct MdxDict {
    storage: DictStorage,
    #[borrows(storage)]
    #[covariant]
    inner: mdx::Dict<&'this [u8]>,
}

impl MdxDict {
    async fn create<P>(path: P) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let storage = fs::read(&path).await?;
        Ok(MdxDictTryBuilder {
            storage,
            inner_builder: |storage| {
                mdx::parse(storage.as_slice())
                    .context(format!("Failed to parse mdx: {}", path.as_ref().display()))
            },
        }
        .try_build()?)
    }

    async fn query(&mut self, q: &str) -> Vec<String> {
        self.with_inner_mut(|inner| {
            inner
                .search(q)
                .iter()
                .filter_map(|item| match &item.1 {
                    mdx::Resource::Text(text) => Some(text.clone()),
                    _ => None,
                })
                .collect()
        })
    }
}

pub struct Dict {
    dicts: Vec<Mutex<MdxDict>>,
}

impl Dict {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs.get::<Config>("dict.mdx").await?;
        let paths = enumerate_paths(cfg.path).await?;

        Ok(Res::new(Self {
            dicts: fetch_dict(paths)
                .await?
                .into_iter()
                .map(|v| Mutex::new(v))
                .collect(),
        }))
    }

    pub async fn query(&self, q: &str) -> QueryResult {
        let mut tasks = Vec::new();
        for dict in &self.dicts {
            tasks.push(async move {
                let mut dict = dict.lock().await;
                dict.query(q).await
            });
        }
        QueryResult {
            result: join_all(tasks).await.into_iter().flatten().collect(),
        }
    }
}

async fn fetch_dict(paths: Vec<PathBuf>) -> Result<Vec<MdxDict>, anyhow::Error> {
    let mut tasks = Vec::new();
    for path in paths {
        tasks.push(MdxDict::create(path));
    }
    try_join_all(tasks).await.context("Waiting for fetch dict")
}

async fn enumerate_paths<P>(path: P) -> Result<Vec<PathBuf>, anyhow::Error>
where
    P: AsRef<Path>,
{
    let meta = fs::metadata(&path).await?;
    let mut vec = Vec::new();
    if meta.is_dir() {
        let mut s = ReadDirStream::new(fs::read_dir(&path).await?);
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
        vec.push(path.as_ref().to_path_buf());
    }
    Ok(vec)
}
