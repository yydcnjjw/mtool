use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use mapp::prelude::*;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};

pub struct WGuiWindow<R: tauri::Runtime = Wry> {
    inner: tauri::WebviewWindow<R>,
    pos: RwLock<Option<PhysicalPosition<i32>>>,
    hide_on_unfocus: bool,
}

impl<R: tauri::Runtime> HasDisplayHandle for WGuiWindow<R> {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.app_handle().display_handle()
    }
}

unsafe impl<R: tauri::Runtime> raw_window_handle5::HasRawDisplayHandle for WGuiWindow<R> {
    fn raw_display_handle(&self) -> raw_window_handle5::RawDisplayHandle {
        self.app_handle().raw_display_handle()
    }
}

impl<R: tauri::Runtime> HasWindowHandle for WGuiWindow<R> {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.inner.window_handle()
    }
}

impl<R: tauri::Runtime> Deref for WGuiWindow<R> {
    type Target = tauri::WebviewWindow<R>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<R: tauri::Runtime> WGuiWindow<R> {
    pub async fn new(
        window: tauri::WebviewWindow<R>,
        hide_on_unfocus: bool,
    ) -> Result<Arc<Self>, anyhow::Error> {
        let this = Arc::new(Self {
            inner: window.clone(),
            pos: RwLock::new(None),
            hide_on_unfocus,
        });

        Self::listen_window_event(this.clone());

        Self::wait_for_ready(this.clone()).await?;

        Ok(this)
    }

    async fn wait_for_ready(self: Arc<Self>) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        self.once("window:ready", move |_| {
            let _ = tx.send(());
            debug!("window:ready");
        });
        Ok(rx.await?)
    }

    fn save_position(&self) -> Result<(), anyhow::Error> {
        let mut pos = self.pos.write().unwrap();
        *pos = Some(self.inner.outer_position()?);
        debug!("save position: {:?}", &pos);
        Ok(())
    }

    fn restore_position(&self) -> Result<(), anyhow::Error> {
        let pos = self.pos.read().unwrap();
        if let Some(pos) = pos.as_ref() {
            self.inner.set_position(pos.clone())?;
            debug!("restore position: {:?}", &pos);
        }
        Ok(())
    }

    fn listen_window_event(self: Arc<Self>) {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.inner.on_window_event(move |e| {
            let _ = tx.send(e.clone());
        });

        tokio::spawn(async move {
            while let Some(e) = rx.recv().await {
                if let Err(e) = self.handle_window_event(e) {
                    warn!("{:?}", e);
                }
            }
        });
    }

    fn handle_window_event(&self, e: WindowEvent) -> Result<(), anyhow::Error> {
        match e {
            WindowEvent::Focused(focused) => {
                if !focused && self.hide_on_unfocus {
                    self.hide()?;
                } else {
                    self.show()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn show(&self) -> Result<(), anyhow::Error> {
        if !self.inner.is_visible()? {
            self.inner.show()?;
            self.inner.set_focus()?;
            self.restore_position()?;
        }
        Ok(())
    }

    pub fn hide(&self) -> Result<(), anyhow::Error> {
        if self.inner.is_visible()? {
            self.save_position()?;
            self.inner.hide()?;
        }
        Ok(())
    }
}

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
    Builder::<R>::new("mtool_window")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move {
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
