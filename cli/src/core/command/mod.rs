mod cmder;

use async_trait::async_trait;

pub use cmder::*;

use crate::app::App;

#[async_trait]
pub trait Command {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<()>;
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let rx = app.evbus.subscribe();
    tokio::spawn(async move {
        Commander::run_loop(rx).await;
        log::debug!("commander run loop quit!");
    });
    Ok(())
}
