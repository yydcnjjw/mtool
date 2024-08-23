use clipboard::{ClipboardContext, ClipboardProvider};
use mapp::prelude::*;
use mtool_cmder::{Cmder, CommandBuilder};
use mtool_main_window::wgui::native::{MtoolWindow, StickyWindow};
use tauri::Emitter;

use super::selection_monitor::SelectionMonitor;

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
    window.show().await
}

pub async fn query_dict(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.emit_to(window.label(), "route", format!("/dict/"))?;
    window.show().await
}

pub async fn enable_selection_monitor(monitor: Res<SelectionMonitor>) -> Result<(), anyhow::Error> {
    monitor.run();
    Ok(())
}

pub async fn disable_selection_monitor(
    monitor: Res<SelectionMonitor>,
) -> Result<(), anyhow::Error> {
    monitor.cancel();
    Ok(())
}

pub async fn init(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(
            query_dict_with_clipboard
                .name("dict.query_with_clipboard")
                .descrption("Query dict with clipboard"),
        )
        .add_command(query_dict.name("dict.query").descrption("Query dict"))
        .add_command(
            enable_selection_monitor
                .name("dict.enable_selection_monitor")
                .descrption("Enable selection monitor"),
        )
        .add_command(
            disable_selection_monitor
                .name("dict.disable_selection_monitor")
                .descrption("Disable selection monitor"),
        );
    Ok(())
}
