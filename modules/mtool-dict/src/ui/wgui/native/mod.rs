use async_trait::async_trait;
use clipboard::{ClipboardContext, ClipboardProvider};
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage,
};
use mtool_system::keybinding::Keybinding;
use mtool_wgui::MtoolWindow;
use tauri::Manager;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(
            AppStage::Init,
            init.cond(is_startup_mode(StartupMode::WGui)),
        );
        Ok(())
    }
}

async fn init(keybinding: Res<Keybinding>, cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    keybinding
        .define_global("M-S-d", query_dict_with_clipboard)
        .await?;
    cmder.add_command(query_dict.name("query_dict").desc("query dict"));
    Ok(())
}

async fn query_dict_with_clipboard(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
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

async fn query_dict(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.emit_to(window.label(), "route", format!("/dict/"))?;
    window.show()?;
    Ok(())
}
