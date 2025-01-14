use std::path::PathBuf;

use mapp::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(crate = "mapp::serde")]
pub struct SearchPath {
    pub path: PathBuf,
    pub depth: u8,
}

impl SearchPath {
    pub fn new(path: PathBuf, depth: u8) -> Self {
        Self { path, depth }
    }
}
