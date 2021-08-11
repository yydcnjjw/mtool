use clap::Clap;
use mdict::common::MdResource;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{app::App, error};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clap)]
pub struct Mdict {
    /// query
    #[clap(required(true), index(1))]
    query: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MdictConfig {
    pub dict_path: String,
}

impl MdictConfig {
    async fn list_dict_paths(&self) -> Result<Vec<PathBuf>> {
        println!("{}", self.dict_path);
        let path = Path::new(&self.dict_path);

        Ok(fs::read_dir(path)?
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    mdict: MdictConfig,
}

impl Mdict {
    pub async fn run(&self, app: &App) -> error::Result<()> {
        let config: Config = app.config.get(&"dict".to_string())?;

        let mut handles = Vec::new();

        for path in config.mdict.list_dict_paths().await? {
            let query = self.query.clone();
            handles.push(tokio::spawn(async move {
                match mdict::parse(&path) {
                    Ok(mut md) => {
                        md.search(&query)
                            .iter()
                            .filter_map(|item| {
                                let text = match &item.1 {
                                    MdResource::Text(text) => text,
                                    _ => {
                                        return None;
                                    }
                                };
                                Some((
                                    item.0.clone(),
                                    format!("<div>{} ----------</div>{}", item.0, text),
                                ))
                            })
                            .for_each(|item| {
                                termimad::print_inline(html2md::parse_html(&item));
                            });
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
