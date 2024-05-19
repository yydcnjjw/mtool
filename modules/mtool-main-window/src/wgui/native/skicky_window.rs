use std::{ops::Deref, sync::Arc};

use mapp::prelude::*;
use mtool_wgui::{WGuiWindow, WindowDataBind};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};
use tokio::sync::oneshot;
use tracing::warn;

use crate::wgui::generic::SKICKY_WINDOW_LABEL;

#[derive(Clone)]
pub struct SkickyWindow<R: tauri::Runtime = Wry>(Arc<WGuiWindow<R>>);

impl<R: tauri::Runtime> SkickyWindow<R> {
    async fn new(app: AppHandle<R>) -> Result<Self, anyhow::Error> {
        let win = WebviewWindowBuilder::new(
            &app,
            SKICKY_WINDOW_LABEL,
            WebviewUrl::App("index.html".into()),
        )
        .title(SKICKY_WINDOW_LABEL)
        .transparent(true)
        .decorations(false)
        .resizable(true)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(true)
        // TODO: disable shadow for transparent
        .shadow(false)
        .build()
        .expect(&format!("create {} window failed", SKICKY_WINDOW_LABEL));

        Ok(Self(
            WGuiWindow::<R>::new(win, cfg!(not(debug_assertions))).await?,
        ))
    }
}

impl<R: tauri::Runtime> Deref for SkickyWindow<R> {
    type Target = WGuiWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    win_tx: oneshot::Sender<Res<SkickyWindow<R>>>,
) -> Result<(), anyhow::Error> {
    let app = app.clone();
    tokio::spawn(async move {
        match SkickyWindow::<R>::new(app).await {
            Ok(win) => {
                win.bind(win.clone());
                let _ = win_tx.send(Res::new(win));
            }
            Err(e) => warn!("{:?}", e),
        }
    });
    Ok(())
}
