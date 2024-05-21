use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use tauri::{Manager, PhysicalPosition, WindowEvent, Wry};
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
    pub async fn new_and_wait_for_ready(
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

    pub fn new(
        window: tauri::WebviewWindow<R>,
        hide_on_unfocus: bool,
    ) -> Result<Arc<Self>, anyhow::Error> {
        let this = Arc::new(Self {
            inner: window.clone(),
            pos: RwLock::new(None),
            hide_on_unfocus,
        });

        Self::listen_window_event(this.clone());

        Ok(this)
    }

    pub async fn wait_for_ready(self: Arc<Self>) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        let label = self.label().to_owned();
        self.once("window:ready", move |_| {
            let _ = tx.send(());
            debug!("{} window:ready", label);
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
