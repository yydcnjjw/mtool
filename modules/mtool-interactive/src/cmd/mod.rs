pub mod command;

use mapp::prelude::*;
use mtool_core::AppStage;
use mtool_wgui::WebStage;

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WebStage::Init, command::web_init);

        Ok(())
    }
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(AppStage::Init, command::init);
        Ok(())
    }
}
