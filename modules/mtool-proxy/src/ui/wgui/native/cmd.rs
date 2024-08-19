use mapp::prelude::*;
use mtool_main_window::wgui::native::sticky::window::StickyWindow;
use tauri::Emitter;

pub async fn show_stats(window: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    window.emit_to(window.label(), "route", format!("/proxy"))?;
    window.show()?;
    Ok(())
}
