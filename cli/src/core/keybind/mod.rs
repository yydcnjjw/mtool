mod kbd;
mod kber;
mod kbnode;
mod kbdispatcher;

use thiserror::Error;

pub use kber::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let mut rx = app.evbus.subscribe();

    

    Ok(())
}
