use crate::app::App;

pub mod evbus;
pub mod config;

mod service;
mod sysev;

// pub mod command;
mod keybind;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    service::module_load(app).await?;
    sysev::module_load(app).await?;
    keybind::module_load(app).await?;
    Ok(())
}
