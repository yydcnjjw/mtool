pub mod command;

use mapp::{anyhow, prelude::*, CreateLocalOnceTaskDescriptor};
use mtool_main_window::wgui::generic::MTOOL_WINDOW_LABEL;
use mtool_wgui::{is_window, WebStage};

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(
            WebStage::Init,
            command::web_init.cond(is_window(MTOOL_WINDOW_LABEL)),
        );
        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        use mtool_wgui::WGuiStage;
        ctx.schedule()
            .add_once_task(WGuiStage::AfterInit, command::init);
        Ok(())
    }
}
