use std::path::Path;

use mapp::prelude::*;
use mtool_interactive::{Completion, CompletionArgs};
use tauri::AppHandle;
use tokio::fs;

use super::PdfViewerWindow;

async fn list_file<P: AsRef<Path>>(dir: P) -> Result<Vec<String>, anyhow::Error> {
    if !fs::try_exists(&dir).await? {
        return Ok(Vec::new());
    }
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

pub async fn open_pdf(app_handle: Res<AppHandle>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    let path: String = match c
        .complete_read(
            CompletionArgs::new(|completed: &str| {
                let completed = completed.to_string();
                async move { list_file(completed).await }
            })
            .prompt("Open pdf: ")
            .hide_window(),
        )
        .await?
    {
        Some(v) => v,
        None => return Ok(()),
    };

    let win = PdfViewerWindow::new((*app_handle).clone()).await?;

    win.open_file(&path)?;
    win.show()?;

    Ok(())
}
