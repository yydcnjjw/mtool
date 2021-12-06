use crate::app::App;

pub mod command;
pub mod config;
pub mod evbus;
pub mod keybind;
mod sysev;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    sysev::module_load(app).await?;
    config::module_load(app).await?;
    keybind::module_load(app).await?;
    command::module_load(app).await?;
    Ok(())
}
