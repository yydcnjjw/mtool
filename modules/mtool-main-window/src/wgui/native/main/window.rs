use std::{ops::Deref, sync::Arc};

use mtool_wgui::WGuiWindow;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};

use crate::wgui::generic::MTOOL_WINDOW_LABEL;

#[derive(Clone)]
pub struct MtoolWindow<R: tauri::Runtime = Wry>(Arc<WGuiWindow<R>>);

impl<R: tauri::Runtime> MtoolWindow<R> {
    pub(crate) async fn new(app: AppHandle<R>) -> Result<Self, anyhow::Error> {
        let win = WebviewWindowBuilder::new(&app, MTOOL_WINDOW_LABEL, WebviewUrl::App("".into()))
            .title(MTOOL_WINDOW_LABEL)
            .transparent(true)
            .visible(false)
            // .transparent(false)
            // .visible(true)
            .decorations(false)
            .resizable(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .focused(true)
            // TODO: disable shadow for transparent
            .shadow(false)
            .build()
            .expect("create mtool window failed");

        Ok(Self(WGuiWindow::<R>::new_and_wait_for_ready(win).await?))
    }
}

impl<R: tauri::Runtime> Deref for MtoolWindow<R> {
    type Target = WGuiWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
