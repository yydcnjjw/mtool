use super::{
    entities::{dict, prelude::Dict as ECDict},
    view,
};
use anyhow::Context;
use mapp::provider::Res;
use mtool_core::ConfigStore;
use sea_orm::*;
use serde::Deserialize;
use std::path::Path;

impl From<dict::Model> for view::QueryResult {
    fn from(word: dict::Model) -> Self {
        let dict::Model {
            id: _,
            sw: _,
            word,
            phonetic,
            definition,
            translation,
            pos,
            collins,
            oxford,
            tag,
            bnc,
            frq,
            exchange,
            detail,
            audio,
        } = word;
        Self {
            word,
            phonetic: phonetic.filter(|v| !v.is_empty()),
            definition: definition.map_or(Vec::new(), |v| {
                v.lines()
                    .filter_map(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect()
            }),
            translation: translation.map_or(Vec::new(), |v| {
                v.lines()
                    .filter_map(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect()
            }),
            pos: pos.map_or(Vec::new(), |v| {
                v.split("/")
                    .filter_map(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect()
            }),
            collins,
            oxford,
            tag: tag.map_or(Vec::new(), |v| {
                v.split_whitespace()
                    .filter_map(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect()
            }),
            bnc,
            frq,
            exchange: exchange.map_or(Vec::new(), |v| {
                v.split("/")
                    .filter_map(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect()
            }),
            detail: detail.filter(|v| !v.is_empty()),
            audio: audio.filter(|v| !v.is_empty()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    path: String,
}

pub struct Dict {
    db: DatabaseConnection,
}

impl Dict {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs.get::<Config>("dict.ecdict").await?;
        Ok(Res::new(Self::new(&cfg.path).await?))
    }

    async fn new<P>(path: P) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let db = Database::connect(format!("sqlite://{}", path.display()))
            .await
            .context(format!("Failed to connect {}", path.display()))?;

        Ok(Self { db })
    }

    pub async fn query(&self, text: &str) -> Result<view::QueryResult, anyhow::Error> {
        ECDict::find()
            .filter(
                dict::Column::Word
                    .eq(text)
                    .or(dict::Column::Sw.like(&format!("{}%", text))),
            )
            .order_by(dict::Column::Sw, Order::Asc)
            .one(&self.db)
            .await?
            .context(format!("Failed to query: {}", text))
            .map(|v| view::QueryResult::from(v))
    }

    #[allow(unused)]
    async fn find(&self, text: &str, limit: u64) -> Result<Vec<view::QueryResult>, anyhow::Error> {
        Ok(ECDict::find()
            .filter(
                dict::Column::Word
                    .eq(text)
                    .or(dict::Column::Sw.like(&format!("{}%", text))),
            )
            .order_by(dict::Column::Sw, Order::Asc)
            .limit(limit)
            .all(&self.db)
            .await
            .context(format!("Failed to find: {}", text))?
            .into_iter()
            .map(|v| view::QueryResult::from(v))
            .collect())
    }
}
