mod window;

use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::CmdlineStage;
use mtool_wgui::{Builder, WGuiStage};

use self::window::PdfViewerWindow;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WGuiStage::Setup, setup)
            .add_once_task(CmdlineStage::AfterInit, register_command);
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(window::init(injector))))?;
    Ok(())
}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(open_pdf.name("open_pdf"));
    Ok(())
}

async fn open_pdf(window: Res<PdfViewerWindow>) -> Result<(), anyhow::Error> {
    window.emit("route", "/pdfviewer")?;
    window.show()?;
    Ok(())
}
