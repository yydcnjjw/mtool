mod window;

use mtool_core::AppStage;
use mtool_system::keybinding::Keybinding;
use mtool_wgui::{Builder, WGuiStage};

use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule, CreateOnceTaskDescriptor,
};

use crate::service::{is_runnable, ProxyService};

use self::window::{hide_window, show_window};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WGuiStage::Setup, setup.cond(is_runnable))
            .add_once_task(AppStage::Init, register_keybinding.cond(is_runnable));
        Ok(())
    }
}

async fn setup(
    builder: Res<Builder>,
    service: Res<ProxyService>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(window::init(service, injector))))?;
    Ok(())
}

async fn register_keybinding(keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
    keybinding.define_global("M-A-p", show_window).await?;
    keybinding.define_global("M-A-S-p", hide_window).await?;
    Ok(())
}
