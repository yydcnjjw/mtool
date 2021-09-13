use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    App(#[from] anyhow::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
