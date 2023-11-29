mod cmd;
mod pdf_document;
mod pdf_page;
mod pdf_viewer;
mod window;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        use windows::*;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        use linux::*;
    }
}

use async_trait::async_trait;
use cmd::open_pdf;
use mapp::{prelude::*, CreateOnceTaskDescriptor};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{
    config::{not_startup_mode, StartupMode},
    CmdlineStage,
};
use mtool_wgui::{Builder, WGuiStage};
pub use window::PdfViewerWindow;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WGuiStage::Setup, setup)
            .add_once_task(
                CmdlineStage::AfterInit,
                register_command.cond(not_startup_mode(StartupMode::Cli)),
            );
        Ok(())
    }
}

async fn setup(builder: Res<Builder>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(window::init())))
}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(open_pdf.name("open_pdf"));
    Ok(())
}
