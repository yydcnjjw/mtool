mod plugin;

use mtool_wgui::{Builder, GuiStage};

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};

use crate::proxy::ProxyApp;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(GuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, proxy_app: Res<ProxyApp>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(plugin::init(proxy_app))))?;
    Ok(())
}
