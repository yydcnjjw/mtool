use async_trait::async_trait;

use crate::app::App;

#[async_trait]
pub trait Module {
    async fn load(&mut self) -> anyhow::Result<()>;
    async fn unload(&mut self) -> anyhow::Result<()>;
}

