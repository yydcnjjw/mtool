use std::ops::Deref;

use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State,
};

use crate::wgui::generic::hotkey::HotkeyMap;

#[command]
async fn get_hotkeys(
    keybindings: State<'_, HotkeyMap>,
) -> Result<HotkeyMap, serde_error::Error> {
    Ok(keybindings.deref().clone())
}

pub(crate) fn init<R: tauri::Runtime>(keybindings: HotkeyMap) -> TauriPlugin<R> {
    Builder::<R>::new("mtool-main-window")
        .setup(move |app, _| {
            app.manage(keybindings);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_hotkeys])
        .build()
}
