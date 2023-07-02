use std::{ops::Deref, sync::Arc};

use mapp::provider::{Injector, Res};
use mtool_wgui::WGuiWindow;
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    WindowBuilder, WindowUrl, Wry,
};

pub struct PdfViewerWindow(Arc<WGuiWindow>);

impl PdfViewerWindow {
    fn new(app: tauri::AppHandle) -> Self {
        Self(WGuiWindow::new(
            WindowBuilder::new(&app, "mtool-pdfviewer", WindowUrl::App("/pdfviewer".into()))
                .title("mtool-pdfviewer")
                .transparent(false)
                .decorations(false)
                .resizable(true)
                .skip_taskbar(false)
                .visible(true)
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

pub(crate) fn init(injector: Injector) -> TauriPlugin<Wry> {
    Builder::new("pdfviewer")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move { injector.insert(Res::new(PdfViewerWindow::new(app))) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .build()
}
