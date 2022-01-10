#[cfg(feature = "service")]
mod config;

#[cfg(feature = "service")]
pub use config::Config;

use thiserror::Error;
use toml::Value;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type SerdeResult<T> = std::result::Result<T, serde_error::Error>;

#[mrpc::service(message(serde, debug))]
pub trait Service {
    fn get_value(key: String) -> SerdeResult<Value>;
}
