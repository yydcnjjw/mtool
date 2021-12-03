use crate::app::App;

pub mod config;
pub mod evbus;

mod sysev;

// pub mod command;
pub mod service;
pub mod keybind;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    // service::module_load(app).await?;
    sysev::module_load(app).await?;
    keybind::module_load(app).await?;
    Ok(())
}
