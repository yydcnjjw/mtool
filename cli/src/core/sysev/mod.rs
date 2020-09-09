use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::app::App;

use super::evbus::{post, Event, Sender};

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = app.evbus.sender();
    tokio::task::spawn_blocking(move || {
        let sender = sender.clone();
        if let Err(e) = sysev::run_loop(|e| {
            if let Err(e) = post(&sender, e) {
                log::warn!("{}", e);
            }
        }) {
            log::error!("{}", e);
        }
    });
    Ok(())
}
