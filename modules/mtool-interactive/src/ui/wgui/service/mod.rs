pub mod completion;

pub use completion::Completion;

use mtool_wgui::{Builder, WGuiStage};

use async_trait::async_trait;
use mapp::prelude::*;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(WGuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(completion::init())))?;

    injector.construct_once(Completion::construct);
    Ok(())
}
