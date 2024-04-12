pub mod command;

use mapp::prelude::*;
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

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        use mtool_core::AppStage;
        ctx.schedule().add_once_task(AppStage::Init, command::init);
        Ok(())
    }
}
