use mapp::prelude::*;
use mtauri_sys::window::Window;
use mtool_cmder::{CommandBuilder, LocalCmder};

async fn hide_window() -> Result<(), anyhow::Error> {
    Window::current()
        .unwrap()
        .hide()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

pub async fn init(cmder: Res<LocalCmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(hide_window.name("window.hide"));
    Ok(())
}
