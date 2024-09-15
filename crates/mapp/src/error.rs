use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize module {0}")]
    ModuleInit(&'static str, #[source] anyhow::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
