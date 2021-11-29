mod cmder;

use async_trait::async_trait;

pub use cmder::*;

#[async_trait]
pub trait Command {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<()>;
}
