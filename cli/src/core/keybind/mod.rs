mod kbd;
mod kbdispatcher;
mod kber;
mod kbnode;

use thiserror::Error;

use crate::app::App;

use self::kber::KeyBindinger;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let tx = app.evbus.sender();
    let mut rx = app.evbus.subscribe();

    tokio::spawn(async move {
        KeyBindinger::run_loop(tx.clone(), rx).await;
    });

    Ok(())
}
