use async_trait::async_trait;

use crate::app::App;

use super::evbus::{post, Sender};

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = app.evbus.sender();
    tokio::task::spawn_blocking(move || {
        // TODO:
        // sysev::run_loop(|e| post(&sender, e));
    });
    Ok(())
}
