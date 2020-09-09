pub mod api;
pub mod parser;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    NetRequest(#[from] reqwest::Error),
    #[error("HJ dict not found: {0}")]
    NotFound(String),
    #[error("HJ dict word suggestion: {0:?}")]
    WordSuggestion(Vec<String>),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
