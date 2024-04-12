use std::ops::Deref;

use tauri::{command, AppHandle, Manager, State};

use crate::wgui::generic::hotkey::HotkeyMap;

#[command]
pub async fn get_hotkeys(keybindings: State<'_, HotkeyMap>) -> Result<HotkeyMap, serde_error::Error> {
    Ok(keybindings.deref().clone())
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    keybindings: HotkeyMap,
) -> Result<(), anyhow::Error> {
    app.manage(keybindings);
    Ok(())
}
