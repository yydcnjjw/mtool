use clipboard::{ClipboardContext, ClipboardProvider};
use mapp::prelude::*;
use mtool_main_window::wgui::native::{StickyWindow, MtoolWindow};
use tauri::Emitter;

pub async fn query_dict_with_clipboard(window: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    let mut context: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| anyhow::anyhow!("Failed to get Clipboard: {}", e.to_string()))?;

    let text = match context.get_contents() {
        Ok(v) => v
            .split_ascii_whitespace()
            .next()
            .map_or(String::default(), |v| v.to_string()),
        Err(_) => "".into(),
    };
    window.emit_to(
        window.label(),
        "route",
        format!("/dict/{}", text.to_lowercase()),
    )?;
    window.show()?;
    Ok(())
}

pub async fn query_dict(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.emit_to(window.label(), "route", format!("/dict/"))?;
    window.show()?;
    Ok(())
}
