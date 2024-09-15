mod dict_query_view;

use mapp::{anyhow, prelude::*, CreateLocalOnceTaskDescriptor};
use mtool_main_window::wgui::generic::MTOOL_WINDOW_LABEL;
use mtool_wgui::prelude::*;

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WebStage::Init, init.cond(is_window(MTOOL_WINDOW_LABEL)));
        Ok(())
    }
}

async fn init(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/dict/:query", dict_query_view::render);
    router.add("/dict/", dict_query_view::render);
    Ok(())
}
