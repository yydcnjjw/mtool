use std::{ops::Deref, sync::Arc};

use mapp::prelude::*;
use mtool_wgui::WGuiWindow;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, WebviewUrl, WebviewWindowBuilder, Wry,
};
use tokio::sync::oneshot;
use tracing::warn;

pub struct MtoolWindow<R: tauri::Runtime = Wry>(Arc<WGuiWindow<R>>);

impl<R: tauri::Runtime> MtoolWindow<R> {
    async fn new(app: AppHandle<R>) -> Result<Self, anyhow::Error> {
        let win = WebviewWindowBuilder::new(&app, "mtool", WebviewUrl::App("index.html".into()))
            .title("mtool")
            .transparent(true)
            .decorations(false)
            .resizable(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(true)
            // TODO: disable shadow for transparent
            .shadow(false)
            .build()
            .expect("create mtool window failed");
        Ok(Self(
            WGuiWindow::<R>::new(
                win, true, // cfg!(not(debug_assertions))
            )
            .await?,
        ))
    }
}

impl<R: tauri::Runtime> Deref for MtoolWindow<R> {
    type Target = WGuiWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn init<R: tauri::Runtime>(
    win_tx: oneshot::Sender<Res<MtoolWindow<R>>>,
) -> TauriPlugin<R> {
    Builder::<R>::new("mtool-main-window")
        .setup(move |app, _| {
            let app = app.clone();
            tokio::spawn(async move {
                match MtoolWindow::<R>::new(app).await {
                    Ok(win) => {
                        let _ = win_tx.send(Res::new(win));
                    }
                    Err(e) => warn!("{:?}", e),
                }
            });
            Ok(())
        })
        .build()
}
