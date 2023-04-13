pub mod completion;
mod output;

pub use completion::Completion;

use mtool_wgui::{Builder, GuiStage};
pub use output::OutputDevice;

use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(GuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(completion::init()).plugin(output::init())))?;

    injector
        .construct_once(Completion::new)
        .construct_once(OutputDevice::new);
    Ok(())
}
