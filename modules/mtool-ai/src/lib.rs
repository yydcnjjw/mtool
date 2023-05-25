mod model;

use async_trait::async_trait;
use mapp::{AppContext, AppModule};

pub use model::llama_chat::LLamaChat;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(LLamaChat::construct);
        Ok(())
    }
}
