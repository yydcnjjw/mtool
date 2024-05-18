mod cmd;
mod hotkey;
mod window;

use mapp::prelude::*;
use mtool_cmder::Cmder;
use mtool_core::{AppStage, ConfigStore};
use mtool_system::keybinding::Keybinding;
use mtool_wgui::{Builder, WGuiStage};
use tauri::generate_handler;
use tokio::sync::oneshot;

pub use window::*;

use super::generic::hotkey::{Hotkey, HotkeyMap};

pub(crate) struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WGuiStage::Setup, setup_plugin)
            .add_once_task(WGuiStage::Setup, cmd::init)
            .add_once_task(AppStage::Init, setup_global_hotkey);

        Ok(())
    }
}

async fn setup_global_hotkey(
    cs: Res<ConfigStore>,
    keybinding: Res<Keybinding>,
) -> Result<(), anyhow::Error> {
    if let Ok(ghkm) = cs.get::<HotkeyMap>("wgui.global_hotkey").await {
        for Hotkey { command, kbd, .. } in ghkm.0 {
            keybinding
                .define_global(&kbd, move |cmder: Res<Cmder>, injector: Injector| {
                    let command = command.clone();
                    async move {
                        if let Some(cmd) = cmder.get_command_with_name(&command) {
                            cmd.exec(&injector).await
                        } else {
                            Ok(())
                        }
                    }
                })
                .await?;
        }
    }

    Ok(())
}

async fn setup_plugin(
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
