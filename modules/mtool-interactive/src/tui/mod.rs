mod output;

use async_trait::async_trait;
use mapp::{AppContext, AppModule};
pub use output::OutputDevice;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector();

        app.schedule();

        Ok(())
    }
}
