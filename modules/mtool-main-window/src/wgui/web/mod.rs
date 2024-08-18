mod cmd;
pub mod hotkey;
mod sticky;

use mapp::prelude::*;
use mtool_wgui::prelude::*;
// use yew::prelude::*;

pub(crate) struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, app: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WebStage::Init, hotkey::register)
            .add_once_task(WebStage::Init, sticky::register);
        Ok(())
    }
}
