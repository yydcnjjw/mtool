pub mod app;
pub mod config;
pub mod convert;

use cloud_api::tencent;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Tencent(#[from] tencent::Error),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("xclip take stdio")]
    TakeStdio,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
