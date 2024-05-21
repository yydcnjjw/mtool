use std::{ops::Deref, sync::Arc};

use mapp::prelude::*;
use mtool_wgui::{WGuiWindow, WindowDataBind};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};
use tokio::sync::oneshot;
use tracing::warn;

use crate::wgui::generic::STICKY_WINDOW_LABEL;

#[derive(Clone)]
pub struct StickyWindow<R: tauri::Runtime = Wry>(Arc<WGuiWindow<R>>);

impl<R: tauri::Runtime> StickyWindow<R> {
    async fn new(app: AppHandle<R>) -> Result<Self, anyhow::Error> {
        let win = WebviewWindowBuilder::new(
            &app,
            STICKY_WINDOW_LABEL,
            WebviewUrl::App("".into()),
        )
        .title(STICKY_WINDOW_LABEL)
        .transparent(false)
        .decorations(false)
        .resizable(true)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(false)
        // TODO: disable shadow for transparent
        .shadow(false)
        .build()
        .expect(&format!("create {} window failed", STICKY_WINDOW_LABEL));

        Ok(Self(
            WGuiWindow::<R>::new_and_wait_for_ready(win, false).await?,
        ))
    }
}

impl<R: tauri::Runtime> Deref for StickyWindow<R> {
    type Target = WGuiWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    win_tx: oneshot::Sender<Res<StickyWindow<R>>>,
) -> Result<(), anyhow::Error> {
    let app = app.clone();
    tokio::spawn(async move {
        match StickyWindow::<R>::new(app).await {
            Ok(win) => {
                win.bind(win.clone());
                let _ = win_tx.send(Res::new(win));
            }
            Err(e) => warn!("{:?}", e),
        }
    });
    Ok(())
}
