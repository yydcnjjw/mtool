use async_trait::async_trait;

use crate::app::App;

use super::service::Service;
struct SysevService {}

#[async_trait]
impl Service for SysevService {
    async fn run_loop(&self) {
        
    }
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = app.evbus.sender();
    AddService::post(sender, Arc::new(SysevService {})).await?;
    Ok(())
}
