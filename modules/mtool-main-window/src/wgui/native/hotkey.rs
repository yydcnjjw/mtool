use std::ops::Deref;

use anyhow::Context;
use mapp::prelude::*;
use mtool_cmder::Cmder;
use tauri::{command, AppHandle, Manager, State};

use crate::wgui::generic::hotkey::HotkeyMap;

#[command]
pub async fn get_hotkeys(
    keybindings: State<'_, HotkeyMap>,
) -> Result<HotkeyMap, serde_error::Error> {
    Ok(keybindings.deref().clone())
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

    cmd.exec(&injector)
        .await
        .map_err(|e| serde_error::Error::new(&*e))?;

    Ok(())
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    keybindings: HotkeyMap,
    injector: Injector,
    cmder: Res<Cmder>,
) -> Result<(), anyhow::Error> {
    app.manage(keybindings);
    app.manage(injector);
    app.manage(cmder);
    Ok(())
}
