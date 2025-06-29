mod cmd;
mod plugin;
mod selection_monitor;

use mapp::{anyhow, prelude::*};
use mtool_wgui::WGuiStage;
use selection_monitor::SelectionMonitor;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WGuiStage::Setup, plugin::setup)
            .add_once_task(WGuiStage::AfterInit, cmd::init);

        app.injector().construct_once(SelectionMonitor::construct);
        Ok(())
    }
}
