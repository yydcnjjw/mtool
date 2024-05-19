mod cmd;
pub mod hotkey;

use mapp::prelude::*;
use mtool_wgui::WebStage;

pub(crate) struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, app: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WebStage::Init, hotkey::register);
            // .add_once_task(WebStage::Init, cmd::init)
        Ok(())
    }
}
