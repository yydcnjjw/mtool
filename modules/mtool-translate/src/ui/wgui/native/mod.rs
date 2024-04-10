use mapp::{prelude::*, CreateOnceTaskDescriptor};
use mtool_cmder::{Cmder, CommandBuilder};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage,
};
use mtool_main_window::wgui::native::MtoolWindow;
use tauri::Manager;

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

async fn init(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(
        text_translate
            .name("text_translate")
            .descrption("text translate"),
    );
    Ok(())
}

pub async fn text_translate(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.emit_to(window.label(), "route", "/translate")?;
    window.show()?;
    Ok(())
}
