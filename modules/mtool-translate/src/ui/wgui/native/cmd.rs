use mapp::prelude::*;
use mtool_main_window::wgui::native::MtoolWindow;
use tauri::Manager;

pub async fn text_translate(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.emit_to(window.label(), "route", "/translate")?;
    window.show()?;
    Ok(())
}
