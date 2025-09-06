use mapp::{anyhow, prelude::*};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, _ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
