mod plugin;

use mtool_core::AppStage;
use mtool_system::keybinding::Keybinding;
use mtool_wgui::{Builder, GuiStage};

use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule, CreateOnceTaskDescriptor,
};

use crate::{is_runnable, proxy::ProxyApp};

use self::plugin::{hide_window, show_window};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(GuiStage::Setup, setup.cond(is_runnable))
            .add_once_task(AppStage::Init, register_keybinding.cond(is_runnable));
        Ok(())
    }
}

async fn setup(
    builder: Res<Builder>,
    proxy_app: Res<ProxyApp>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(plugin::init(proxy_app, injector))))?;
    Ok(())
}

async fn register_keybinding(keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
    keybinding.define_global("M-A-p", show_window).await?;
    keybinding.define_global("M-A-S-p", hide_window).await?;
    Ok(())
}
