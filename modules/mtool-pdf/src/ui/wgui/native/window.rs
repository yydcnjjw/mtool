use std::{ops::Deref, sync::Arc};

use mapp::provider::Res;
use mtool_wgui::WGuiWindow;
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    WindowBuilder, WindowUrl, Wry,
};

use tokio::sync::oneshot;
use tracing::warn;

use super::{pdf_viewer::PdfViewer, Renderer, RendererBuilder};

pub struct PdfViewerWindow {
    win: Arc<WGuiWindow>,
    _renderer: Renderer,
    _pdf_viewer: PdfViewer,
}

impl PdfViewerWindow {
    const WINDOW_LABEL: &'static str = "mtool-pdfviewer";

    async fn new(app: tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let win = WGuiWindow::new(
            {
                #[allow(unused_mut)]
                let mut builder = WindowBuilder::new(
                    &app,
                    Self::WINDOW_LABEL,
                    WindowUrl::App("/index.html".into()),
                )
                .title(Self::WINDOW_LABEL)
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
            },
            false,
        );

        {
            let window = win.clone();
            win.on_window_event(move |e| match e {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                _ => {}
            });
        }

        let pdf_viewer = PdfViewer::new(win.clone()).await?;

        let renderer = {
            let viewer = pdf_viewer.clone();
            RendererBuilder::new(win.clone())
                .add_draw_hook(move |c| viewer.draw(c.canvas))
                .build()
                .await?
        };

        Ok(Self {
            win,
            _renderer: renderer,
            _pdf_viewer: pdf_viewer,
        })
    }
}

impl Deref for PdfViewerWindow {
    type Target = WGuiWindow;

    fn deref(&self) -> &Self::Target {
        &self.win
    }
}

pub(crate) fn init(tx: oneshot::Sender<Res<PdfViewerWindow>>) -> TauriPlugin<Wry> {
    Builder::new("pdfviewer")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move {
                match PdfViewerWindow::new(app).await {
                    Ok(win) => {
                        let _ = tx.send(Res::new(win));
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .build()
}
