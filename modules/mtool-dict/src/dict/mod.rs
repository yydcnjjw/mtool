mod entities;

use entities::{prelude::*, *};

use std::{fmt, path::Path};

use anyhow::Context;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

struct DictDB {
    db: DatabaseConnection,
}

#[derive(Serialize, Deserialize)]
struct Word {
    word: String,
    phonetic: Option<String>,
    definition: Vec<String>,
    translation: Vec<String>,
    pos: Vec<String>,
    collins: Option<i32>,
    oxford: Option<i32>,
    tag: Vec<String>,
    bnc: Option<i32>,
    frq: Option<i32>,
    exchange: Vec<String>,
    detail: Option<String>,
    audio: Option<String>,
}

impl From<dict::Model> for Word {
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
        Word {
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

impl DictDB {
    async fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + fmt::Display,
    {
        let db = Database::connect(format!("sqlite://{}", path))
            .await
            .context(format!("Failed to connect {}", path))?;

        Ok(Self { db })
    }

    async fn query(&self, text: &str, limit: u64) -> anyhow::Result<Vec<Word>> {
        Ok(Dict::find()
            .filter(
                dict::Column::Word
                    .eq(text)
                    .or(dict::Column::Sw.like(&format!("{}%", text))),
            )
            .order_by(dict::Column::Sw, Order::Asc)
            .limit(limit)
            .all(&self.db)
            .await
            .context(format!("Failed to query: {}", text))?
            .into_iter()
            .map(|v| Word::from(v))
            .collect())
    }
}

#[tauri::command]
async fn query(
    text: String,
    limit: u64,
    dict: tauri::State<'_, DictDB>,
) -> Result<Vec<Word>, String> {
    dict.query(&text, limit)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    path: String,
}

pub async fn init<R: Runtime>(cfg: Config) -> anyhow::Result<TauriPlugin<R>> {
    let dict = DictDB::new(cfg.path).await?;

    Ok(Builder::new("dict")
        .invoke_handler(tauri::generate_handler![query])
        .setup(|app_handle| {
            app_handle.manage(dict);
            Ok(())
        })
        .build())
}
