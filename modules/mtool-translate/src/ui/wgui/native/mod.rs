use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage,
};
use mtool_system::keybinding::Keybinding;
use mtool_wgui::MtoolWindow;

#[derive(Default)]
pub struct Module {}

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
    keybinding.define_global("M-A-S-t", text_translate).await?;
    cmder.add_command(text_translate.name("text_translate").desc("text translate"));
    Ok(())
}

pub async fn text_translate(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.emit("route", "/translate")?;
    window.show()?;
    Ok(())
}
