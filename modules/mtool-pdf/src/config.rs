use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub library: String,
    pub cache_dir: PathBuf,
    pub search_dir: Vec<PathBuf>,
}
