mod cmder;

use std::{any::Any, sync::Arc};

use async_trait::async_trait;

pub use cmder::*;

use crate::app::App;

pub type Output = Arc<dyn Any + Send + Sync>;

#[async_trait]
pub trait Command {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<Output>;
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let rx = app.evbus.subscribe();
    tokio::spawn(async move {
        log::debug!("commander run loop!");
        Commander::run_loop(rx).await;
        log::debug!("commander run loop quit!");
    });
    Ok(())
}
