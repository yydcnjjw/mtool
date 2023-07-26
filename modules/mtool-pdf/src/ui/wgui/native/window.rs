use std::{ops::Deref, sync::Arc};

use mtool_wgui::WGuiWindow;
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    WindowBuilder, WindowUrl, Wry,
};
use tokio::sync::oneshot;

pub struct PdfViewerWindow(Arc<WGuiWindow>);

impl PdfViewerWindow {
    fn new(app: tauri::AppHandle) -> Self {
        Self(WGuiWindow::new(
            WindowBuilder::new(
                &app,
                "mtool-pdfviewer",
                WindowUrl::App("/index.html".into()),
            )
            .title("mtool-pdfviewer")
            .transparent(false)
            .decorations(true)
            .resizable(true)
            .skip_taskbar(false)
            .visible(false)
            .build()
            .expect("create proxy monitor window failed"),
            false,
        ))
    }
}

impl Deref for PdfViewerWindow {
    type Target = WGuiWindow;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn init(tx: oneshot::Sender<PdfViewerWindow>) -> TauriPlugin<Wry> {
    Builder::new("pdfviewer")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move {
                let _ = tx.send(PdfViewerWindow::new(app));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .build()
}
