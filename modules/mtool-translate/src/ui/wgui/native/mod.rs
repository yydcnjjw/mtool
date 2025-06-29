mod cmd;
mod plugin;

use mapp::{anyhow, prelude::*};
use mtool_cmder::{Cmder, CommandBuilder};
use mtool_wgui::WGuiStage;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WGuiStage::Setup, plugin::setup)
            .add_once_task(WGuiStage::AfterInit, init);
        Ok(())
    }
}

async fn init(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(
        cmd::text_translate
            .name("translate.text")
            .descrption("Translate text"),
    );
    Ok(())
}
