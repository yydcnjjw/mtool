mod key;
mod record;

pub mod event;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    XLib(String),
    #[error("{0}")]
    XRecord(String),
    #[error("Init record failed")]
    Init,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
