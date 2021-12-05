mod kbd;
mod kbdispatcher;
mod kber;
mod kbnode;

use thiserror::Error;

use crate::app::App;

use self::{kbdispatcher::KeyBindingDispatcher, kber::KeyBindinger};

pub use kbdispatcher::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let tx = app.evbus.sender();
    let rx = app.evbus.subscribe();
    tokio::spawn(async move {
        KeyBindingDispatcher::run_loop(tx, rx).await;
    });

    let tx = app.evbus.sender();
    let rx = app.evbus.subscribe();
    tokio::spawn(async move {
        KeyBindinger::run_loop(tx, rx).await;
    });

    Ok(())
}
