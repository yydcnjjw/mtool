use std::{
    ops::Deref,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::Context;
use base64::prelude::*;
use mtool_wgui::{WGuiWindow, WindowDataBind};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, WindowBuilder, WindowUrl, Wry,
};
use tracing::{debug, warn};

use crate::ui::wgui::{
    event::{WPdfEvent, WPdfLoadEvent},
    service::{PdfLoadEvent, PdfLoadWorker},
};

use super::{
    pdf_viewer::{PdfEvent, PdfViewer},
    Renderer, RendererBuilder,
};

#[derive(Clone)]
pub struct PdfViewerWindow {
    win: Arc<WGuiWindow>,
    _renderer: Arc<Renderer>,
    pdf_viewer: Arc<PdfViewer>,
}

impl PdfViewerWindow {
    pub fn open_file(&self, path: PathBuf) -> Result<(), anyhow::Error> {
        self.win
            .emit(
                "route",
                format!(
                    "/pdfviewer/{}",
                    BASE64_STANDARD.encode(path.display().to_string())
                ),
            )
            .context("emit route failed")?;
        Ok(())
    }

    pub fn set_pdf_loader(&self, mut loader: PdfLoadWorker) {
        {
            let pdf_viewer = self.pdf_viewer.clone();
            let win = self.win.clone();
            tokio::spawn(async move {
                let mut rx = if let Some(rx) = loader.subscribe() {
                    rx
                } else {
                    return;
                };

                while let Some(e) = rx.recv().await {
                    if let Err(e) = win.emit(
                        "pdf_load",
                        match &e {
                            PdfLoadEvent::DocLoading => WPdfLoadEvent::DocLoading,
                            PdfLoadEvent::DocLoaded(doc) => {
                                WPdfLoadEvent::DocLoaded(doc.info().clone())
                            }
                            PdfLoadEvent::DocStructureLoading => WPdfLoadEvent::DocStructureLoading,
                            PdfLoadEvent::DocStructureLoaded(_) => {
                                WPdfLoadEvent::DocStructureLoaded
                            }
                        },
                    ) {
                        warn!("{:?}", e);
                    }
                    pdf_viewer.notify_event(PdfEvent::PdfLoad(e));
                }
            });
        }
    }

    fn window_index() -> usize {
        static INDEX: AtomicUsize = AtomicUsize::new(0);
        INDEX.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn new(app: tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let win = {
            let label = format!("mtool-pdfviewer-{}", Self::window_index());

            #[allow(unused_mut)]
            let mut builder = WindowBuilder::new(&app, &label, WindowUrl::App("/pdfviewer".into()))
                .title(label)
                .resizable(true)
                .skip_taskbar(false)
                .visible(false)
                .transparent(true)
                .decorations(true)
                .shadow(false)
                .disable_file_drop_handler();

            #[cfg(windows)]
            {
                builder = builder.enable_composition();
            }

            builder.build()?
        };
        let win = WGuiWindow::new(win, false).await?;

        let pdf_viewer = Arc::new(PdfViewer::new(win.inner_size()?).await?);

        let renderer = {
            let viewer = pdf_viewer.clone();
            RendererBuilder::new(win.clone())
                .add_draw_hook(move |c| viewer.draw(c.canvas))
                .build()
                .await?
        };

        let this = Self {
            win: win.clone(),
            _renderer: Arc::new(renderer),
            pdf_viewer,
        };

        this.listen_event();

        win.bind(this.clone());

        Ok(this)
    }

    fn listen_event(&self) {
        {
            let pdf_viewer = self.pdf_viewer.clone();
            self.listen("pdf-event", move |e| {
                match serde_json::from_str::<WPdfEvent>(e.payload()) {
                    Ok(e) => {
                        pdf_viewer.notify_event(PdfEvent::WGui(e));
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                }
            });
        }

        {
            let pdf_viewer = self.pdf_viewer.clone();
            self.on_window_event(move |e| {
                pdf_viewer.notify_event(PdfEvent::Window(e.clone()));
            });
        }
    }
}

impl Deref for PdfViewerWindow {
    type Target = WGuiWindow;

    fn deref(&self) -> &Self::Target {
        &self.win
    }
}

pub(crate) fn init() -> TauriPlugin<Wry> {
    Builder::new("pdfviewer")
        .invoke_handler(tauri::generate_handler![])
        .build()
}
