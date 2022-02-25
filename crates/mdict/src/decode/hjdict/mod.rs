mod api;
mod parser;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Word not found: {0}")]
    NotFound(String),
    #[error("Word suggestion: {0:?}")]
    WordSuggestion(Vec<String>),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub use api::query_jp_dict;
