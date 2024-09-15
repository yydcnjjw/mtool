mod main;
mod plugin;
pub mod sticky;

use mapp::{anyhow, prelude::*, tracing::debug};
use mtool_cmder::Cmder;
use mtool_core::{AppStage, ConfigStore};
use mtool_system::keybinding::Keybinding;
use mtool_wgui::WGuiStage;

use super::generic::hotkey::{Hotkey, HotkeyMap};

pub use main::MtoolWindow;
pub use sticky::StickyWindow;

pub(crate) struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WGuiStage::Setup, plugin::setup)
            .add_once_task(WGuiStage::Setup, main::setup)
            .add_once_task(WGuiStage::Setup, sticky::setup::<tauri::Wry>)
            .add_once_task(AppStage::Init, setup_global_hotkey);

        Ok(())
    }
}

async fn setup_global_hotkey(
    cs: Res<ConfigStore>,
    keybinding: Res<Keybinding>,
) -> Result<(), anyhow::Error> {
    if let Ok(ghkm) = cs.get::<HotkeyMap>("wgui.global.hotkey").await {
        for Hotkey { command, kbd, .. } in ghkm.0 {
            debug!("define {} -> {}", kbd, command);
            keybinding
                .define_global(&kbd, move |cmder: Res<Cmder>, injector: Injector| {
                    let command = command.clone();
                    async move {
                        debug!("execute command with global hotkey: {}", command);
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
