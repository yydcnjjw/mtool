use thiserror::Error;

use crate::{command, config};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] config::Error),
    #[error("{0}")]
    Opts(#[from] clap::Error),
    #[error("{0}")]
    Mdict(#[from] command::mdict::Error),
    #[error("{0}")]
    Ocr(#[from] command::ocr::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
