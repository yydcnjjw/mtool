use std::{fmt, fs, path::Path, sync::Mutex};

use anyhow::Context;
use mdict::decode::mdx;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

struct Dict {
    mdx: Mutex<mdx::Dict<&'static [u8]>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WordDetail {
    word: String,
    detail: String,
}

impl Dict {
    fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + fmt::Display,
    {
        static DATA: OnceCell<Vec<u8>> = OnceCell::new();

        let data = DATA.get_or_try_init(|| {
            fs::read(&path).context(format!("Failed to read dict: {}", path.to_string()))
        })?;
        let mdx = Mutex::new(mdx::parse(data.as_slice()).context("Failed to parse mdx")?);
        Ok(Self { mdx })
    }
}

#[tauri::command]
fn query(input: String, dict: tauri::State<Dict>) -> Result<WordDetail, String> {
    let dict = dict.mdx.lock().unwrap();
    let (word, res) = dict.search(&input).map_err(|e| e.to_string())?;

    match res {
        mdx::Resource::Text(v) => Ok(WordDetail {
            word: word.clone(),
            detail: v.clone(),
        }),
        mdx::Resource::Raw(_) => Err("invalid query".into()),
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("dict")
        .invoke_handler(tauri::generate_handler![query])
        .setup(|app_handle| {
            app_handle.manage(Dict::new(
                "/home/yydcnjjw/.mtool/dict/mdict/concise-bing.mdx",
            )?);
            Ok(())
        })
        .build()
}
