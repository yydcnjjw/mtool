use std::{ops::Deref, sync::Arc};

use mapp::prelude::*;
use mtool_wgui::prelude::*;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry};
use tracing::{debug, warn};

use crate::wgui::generic::STICKY_WINDOW_LABEL;

use super::api;

#[derive(Clone)]
pub struct StickyWindow<R: tauri::Runtime = Wry>(Arc<WGuiWindow<R>>);

impl<R: tauri::Runtime> StickyWindow<R> {
    pub(crate) async fn construct(app: Res<AppHandle<R>>) -> Result<Res<Self>, anyhow::Error> {
        let win = WebviewWindowBuilder::new(
            app.deref(),
            STICKY_WINDOW_LABEL,
            WebviewUrl::App("/sticky".into()),
        )
        .title(STICKY_WINDOW_LABEL)
        .transparent(true)
        .decorations(false)
        .resizable(true)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(false)
        .focused(false)
        .drag_and_drop(false)
        // TODO: disable shadow for transparent
        .shadow(false)
        .build()
        .expect(&format!("create {} window failed", STICKY_WINDOW_LABEL));

        let this = Self(WGuiWindow::<R>::new_and_wait_for_ready(win).await?);
        this.bind(this.clone());

        {
            let emitter = this.base();
            this.on_window_event(move |e| match e {
                WindowEvent::MouseInput { state, button, .. }
                    if matches!(button, tauri::MouseButton::Left)
                        && matches!(state, tauri::ElementState::Released) =>
                {
                    if let Err(e) = api::left_mouse_up(&emitter) {
                        warn!("{:?}", e);
                    }
                }
                _ => {}
            });
        }

        Ok(Res::new(this))
    }
}

impl<R: tauri::Runtime> Deref for StickyWindow<R> {
    type Target = WGuiWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
