use async_trait::async_trait;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Run(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait AsyncOperate: fmt::Display + Send + Sync {
    async fn run(&self) -> Result<()>;
}
