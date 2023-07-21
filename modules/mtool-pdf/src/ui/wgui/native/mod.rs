mod window;

use std::path::Path;

use async_trait::async_trait;
use base64::prelude::*;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{CmdlineStage, ConfigStore};
use mtool_interactive::{Completion, CompletionArgs};
use mtool_wgui::{Builder, WGuiStage};
use tokio::fs;
pub use window::PdfViewerWindow;

use crate::Config;

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
    let tx = injector.construct_oneshot();
    builder.setup(|builder| Ok(builder.plugin(window::init(tx))))
}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(open_pdf.name("open_pdf"));
    Ok(())
}

async fn list_file<P: AsRef<Path>>(dir: P) -> Result<Vec<String>, anyhow::Error> {
    let mut entries = fs::read_dir(dir).await?;
    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file()
            && entry.path().extension().is_some_and(|ext| ext == "pdf")
        {
            files.push(
                entry
                    .path()
                    .into_os_string()
                    .into_string()
                    .map_err(|e| anyhow::anyhow!("convert OsString failed: {:?}", e))?,
            )
        }
    }
    Ok(files)
}

async fn open_pdf(
    window: Res<PdfViewerWindow>,
    c: Res<Completion>,
    // cs: Res<ConfigStore>,
) -> Result<(), anyhow::Error> {
    // let cfg: Config = cs.get("pdf").await?;

    let path: String = match c
        .complete_read(
            CompletionArgs::without_completion()
                // CompletionArgs::new(|completed| async move {
                //     list_file(completed).await
                // })
                .prompt("Open pdf: ")
                .hide_window(),
        )
        .await?
    {
        Some(v) => v,
        None => return Ok(()),
    };

    window.emit(
        "route",
        format!("/pdfviewer/{}", BASE64_STANDARD.encode(path)),
    )?;
    window.show()?;
    Ok(())
}
