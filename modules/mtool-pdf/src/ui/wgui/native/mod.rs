mod cmd;
mod pdf_document;
mod pdf_page;
mod pdf_viewer;
mod window;

mapp::cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        use windows::*;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        use linux::*;
    }
}

use cmd::open_pdf;
use mapp::{anyhow, prelude::*};
use mtool_cmder::{Cmder, CommandBuilder};
use mtool_wgui::{Builder, WGuiStage};
pub use window::PdfViewerWindow;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(WGuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(window::init())))?;

    cmder.add_command(open_pdf.name("pdf.open").descrption("Open pdf from file"));
    Ok(())
}
