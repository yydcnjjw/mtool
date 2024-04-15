mod cmd;
mod plugin;

use mapp::prelude::*;
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
    cmder
        .add_command(
            cmd::query_dict_with_clipboard
                .name("dict.query_with_clipboard")
                .descrption("Query dict with clipboard"),
        )
        .add_command(cmd::query_dict.name("dict.query").descrption("Query dict"));

    Ok(())
}
