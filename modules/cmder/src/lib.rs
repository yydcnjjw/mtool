mod cmder;
mod command;

pub use cmder::Cmder;
use command::AsyncCommand;
pub use command::*;

// use thiserror::Error;

// #[derive(Debug, Error)]
// pub enum Error {
//     #[error(transparent)]
//     Other(#[from] anyhow::Error),
// }

// type Result<T> = std::result::Result<T, Error>;

#[mrpc::service]
pub trait Service {
    async fn add(name: String, cmd: AsyncCommand);
    async fn remove(name: String);
    async fn exec(name: String, args: Vec<String>);
}
