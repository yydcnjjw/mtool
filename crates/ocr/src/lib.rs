mod app;
pub mod config;
mod convert;

// use std::{
//     io::Write,
//     process::{Command, Stdio},
// };

use std::io;

use app::App;
use cloud_api::tencent;
use config::Config;
use thiserror::Error;
// use cxx::{CxxVector, UniquePtr};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Tencent(#[from] tencent::Error),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("xclip take stdio")]
    TakeStdio
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn run(config: Config) -> Result<()> {
    App::new(config).run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
