use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use mapp::provider::{Injector, Res};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, PhysicalPosition, WindowBuilder, WindowEvent, WindowUrl, Wry,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};

pub struct WGuiWindow {
    inner: tauri::Window,
    pos: RwLock<Option<PhysicalPosition<i32>>>,
    hide_on_unfocus: bool,
}

unsafe impl HasRawDisplayHandle for WGuiWindow {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.app_handle().raw_display_handle()
    }
}

unsafe impl HasRawWindowHandle for WGuiWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.inner.raw_window_handle()
    }
}

impl Deref for WGuiWindow {
    type Target = tauri::Window;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl WGuiWindow {
    pub async fn new(
        window: tauri::Window,
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
                self.handle_window_event(e);
            }
        });
    }

    fn handle_window_event(&self, e: WindowEvent) {
        match e {
            WindowEvent::Focused(focused) => {
                if !focused && self.hide_on_unfocus {
                    let _ = self.hide();
                }
            }

            _ => {}
        }
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

pub struct MtoolWindow(Arc<WGuiWindow>);

impl MtoolWindow {
    async fn new(app: AppHandle) -> Result<Self, anyhow::Error> {
        let win = WindowBuilder::new(&app, "mtool", WindowUrl::App("index.html".into()))
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
            WGuiWindow::new(win, cfg!(not(debug_assertions))).await?,
        ))
    }
}

impl Deref for MtoolWindow {
    type Target = WGuiWindow;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn show_window(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.show()
}

pub async fn hide_window(window: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    window.hide()
}

pub(crate) fn init(injector: Injector) -> TauriPlugin<Wry> {
    Builder::new("mtool_window")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move {
                match MtoolWindow::new(app).await {
                    Ok(win) => {
                        injector.insert(Res::new(win));
                    }
                    Err(e) => warn!("{:?}", e),
                }
            });
            Ok(())
        })
        .build()
}
