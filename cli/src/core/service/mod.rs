mod server;

use std::sync::Arc;

use async_trait::async_trait;

use crate::app::App;

use self::server::Server;

pub use server::{AddService, RunAll};

#[async_trait]
pub trait Service {
    async fn run_loop(&self);
}

type DynamicService = Arc<dyn Service + Send + Sync>;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let rx = app.evbus.subscribe();
    tokio::spawn(async move {
        Server::run_loop(rx).await;
    });
    Ok(())
}
