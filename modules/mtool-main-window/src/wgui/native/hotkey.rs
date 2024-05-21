use anyhow::Context;
use mapp::prelude::*;
use mtool_cmder::Cmder;
use mtool_core::ConfigStore;
use tauri::{command, AppHandle, Manager, State};
use tracing::debug;

use crate::wgui::generic::hotkey::HotkeyMap;

#[command]
pub async fn get_hotkeys(
    window: tauri::WebviewWindow,
    injector: State<'_, Injector>,
) -> Result<HotkeyMap, serde_error::Error> {
    Ok(match injector.get::<Res<ConfigStore>>().await {
        Ok(cs) => cs
            .get::<HotkeyMap>(&format!("wgui.{}.hotkey", window.label()))
            .await
            .unwrap_or_default(),
        _ => HotkeyMap::default(),
    })
}

#[command]
pub async fn exec_command(
    cmder: State<'_, Res<Cmder>>,
    injector: State<'_, Injector>,
    command: String,
) -> Result<(), serde_error::Error> {
    let cmd = cmder
        .get_command_with_name(&command)
        .context(format!("{} not found", command))
        .map_err(|e| serde_error::Error::new(&*e))?;

    debug!("execute command with wgui: {}", cmd.get_name());

    cmd.exec(&injector)
        .await
        .map_err(|e| serde_error::Error::new(&*e))?;

    Ok(())
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    injector: Injector,
    cmder: Res<Cmder>,
) -> Result<(), anyhow::Error> {
    app.manage(injector);
    app.manage(cmder);
    Ok(())
}
