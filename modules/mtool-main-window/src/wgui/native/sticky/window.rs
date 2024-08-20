use std::{ops::Deref, sync::Arc};

use mtool_wgui::prelude::*;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};

use crate::wgui::generic::STICKY_WINDOW_LABEL;

#[derive(Clone)]
pub struct StickyWindow<R: tauri::Runtime = Wry>(Arc<WGuiWindow<R>>);

impl<R: tauri::Runtime> StickyWindow<R> {
    pub(crate) async fn new(app: AppHandle<R>) -> Result<Self, anyhow::Error> {
        let win =
            WebviewWindowBuilder::new(&app, STICKY_WINDOW_LABEL, WebviewUrl::App("/sticky".into()))
                .title(STICKY_WINDOW_LABEL)
                .transparent(true)
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
