pub mod action;
pub mod api;
mod message;

use mapp::{
    anyhow,
    thiserror::{self, Error},
};
use message::Request;

#[derive(Error, Debug)]
pub enum Error<T>
where
    T: Send + 'static,
{
    #[error("invoke error: {request:?}, {error}")]
    Invoke { request: Request<T>, error: String },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
