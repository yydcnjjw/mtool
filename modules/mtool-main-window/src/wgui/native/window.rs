use std::{ops::Deref, sync::Arc};

use mapp::prelude::*;
use mtool_wgui::{WGuiWindow, WindowDataBind};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};
use tokio::sync::oneshot;
use tracing::warn;

#[derive(Clone)]
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
            .visible(false)
            // TODO: disable shadow for transparent
            .shadow(false)
            .build()
            .expect("create mtool window failed");

        Ok(Self(
            WGuiWindow::<R>::new(win, cfg!(not(debug_assertions))).await?,
        ))
    }
}

impl<R: tauri::Runtime> Deref for MtoolWindow<R> {
    type Target = WGuiWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    win_tx: oneshot::Sender<Res<MtoolWindow<R>>>,
) -> Result<(), anyhow::Error> {
    let app = app.clone();
    tokio::spawn(async move {
        match MtoolWindow::<R>::new(app).await {
            Ok(win) => {
                win.bind(win.clone());
                let _ = win_tx.send(Res::new(win));
            }
            Err(e) => warn!("{:?}", e),
        }
    });
    Ok(())
}
