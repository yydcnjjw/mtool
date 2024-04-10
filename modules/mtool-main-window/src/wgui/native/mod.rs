mod hotkey;
mod window;

use mapp::prelude::*;
use mtool_core::ConfigStore;
use mtool_wgui::{Builder, WGuiStage};
use tokio::sync::oneshot;

pub use window::*;

use super::generic::hotkey::HotkeyMap;

pub(crate) struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(WGuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(
    builder: Res<Builder>,
    cs: Res<ConfigStore>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    let mtool_win_tx: oneshot::Sender<Res<MtoolWindow>> = injector.construct_oneshot();
    let hkm = cs.get::<HotkeyMap>("wgui.hotkey").await?;
    builder.setup(|builder| {
        Ok(builder
            .plugin(window::init(mtool_win_tx))
            .plugin(hotkey::init(hkm)))
    })?;
    Ok(())
}
