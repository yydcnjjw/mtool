mod hotkey;
mod window;
mod cmd;

use mapp::prelude::*;
use mtool_cmder::Cmder;
use mtool_core::ConfigStore;
use mtool_wgui::{Builder, WGuiStage};
use tauri::generate_handler;
use tokio::sync::oneshot;

pub use window::*;

use super::generic::hotkey::HotkeyMap;

pub(crate) struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(WGuiStage::Setup, setup)
            .add_once_task(WGuiStage::Setup, cmd::init);
        Ok(())
    }
}

async fn setup(
    builder: Res<Builder>,
    cs: Res<ConfigStore>,
    injector: Injector,
    cmder: Res<Cmder>,
) -> Result<(), anyhow::Error> {
    let win_tx: oneshot::Sender<Res<MtoolWindow>> = injector.construct_oneshot();
    let hkm = cs.get::<HotkeyMap>("wgui.hotkey").await?;

    builder.setup(|builder| {
        Ok(builder.plugin(
            tauri::plugin::Builder::<_, ()>::new("mtool-main-window")
                .setup(move |app, _| {
                    window::plugin_setup(app, win_tx)?;
                    hotkey::plugin_setup(app, hkm, injector, cmder)?;
                    Ok(())
                })
                .invoke_handler(generate_handler![hotkey::get_hotkeys, hotkey::exec_command])
                .build(),
        ))
    })?;
    Ok(())
}
