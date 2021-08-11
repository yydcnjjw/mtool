use thiserror::Error;

use crate::config;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] config::Error),
    #[error("{0}")]
    Opts(#[from] clap::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
