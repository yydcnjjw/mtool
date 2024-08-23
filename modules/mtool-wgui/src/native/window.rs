use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use mapp::sync::RwLock;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use tauri::{Listener, Manager, PhysicalPosition, WebviewWindow, WindowEvent, Wry};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};
use windows::Win32::{
    Foundation::*,
    UI::{Shell::*, WindowsAndMessaging::*},
};

pub struct WGuiWindow<R: tauri::Runtime = Wry> {
    inner: tauri::WebviewWindow<R>,
    pos: Arc<RwLock<Option<PhysicalPosition<i32>>>>,

    ignore_menu_event: bool,
}

impl<R: tauri::Runtime> WGuiWindow<R> {
    pub async fn new_and_wait_for_ready(
        window: tauri::WebviewWindow<R>,
    ) -> Result<Arc<Self>, anyhow::Error> {
        let this = Arc::new(Self {
            inner: window.clone(),
            pos: Arc::new(RwLock::new(None)),

            ignore_menu_event: true,
        });

        Self::listen_window_event(this.clone()).await?;

        Self::wait_for_ready(this.clone()).await?;

        Ok(this)
    }

    pub async fn new(
        window: tauri::WebviewWindow<R>,
    ) -> Result<Arc<Self>, anyhow::Error> {
        let this = Arc::new(Self {
            inner: window.clone(),
            pos: Arc::new(RwLock::new(None)),

            ignore_menu_event: true,
        });

        Self::listen_window_event(this.clone()).await?;

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
        let mut pos = self.pos.write();
        *pos = Some(self.inner.outer_position()?);
        debug!("save position: {:?}", &pos);
        Ok(())
    }

    fn restore_position(&self) -> Result<(), anyhow::Error> {
        let pos = self.pos.read();
        if let Some(pos) = pos.as_ref() {
            self.inner.set_position(pos.clone())?;
            debug!("restore position: {:?}", &pos);
        }
        Ok(())
    }

    unsafe extern "system" fn main_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _uidsubclass: usize,
        _dwrefdata: usize,
    ) -> LRESULT {
        if msg == WM_SYSCOMMAND && wparam.0 == SC_KEYMENU as usize {
            return LRESULT(0);
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }

    async fn listen_window_event(self: Arc<Self>) -> Result<(), anyhow::Error> {
        if self.ignore_menu_event {
            let win = self.clone();
            self.run_on_main_thread(move |_| unsafe {
                if !SetWindowSubclass(win.hwnd().unwrap(), Some(Self::main_proc), 8082, 0).as_bool()
                {
                    panic!("SetWindowSubclass failed");
                }
                Ok(())
            })
            .await?;
        }

        let (tx, mut rx) = mpsc::unbounded_channel();

        self.inner.on_window_event(move |e| {
            let _ = tx.send(e.clone());
        });

        tokio::spawn(async move {
            while let Some(e) = rx.recv().await {
                if let Err(e) = self.handle_window_event(e).await {
                    warn!("{:?}", e);
                }
            }
        });

        Ok(())
    }

    async fn handle_window_event(&self, e: WindowEvent) -> Result<(), anyhow::Error> {
        match e {
            _ => {}
        }
        Ok(())
    }

    pub async fn run_on_main_thread<
        O,
        F: FnOnce(WGuiWindow<R>) -> Result<O, anyhow::Error> + Send + 'static,
    >(
        &self,
        f: F,
    ) -> Result<O, anyhow::Error>
    where
        O: Send + 'static,
    {
        let win = self.clone();
        let (tx, rx) = oneshot::channel();
        self.inner.run_on_main_thread(move || {
            let _ = tx.send(f(win));
        })?;

        rx.await
            .map_err(|_| anyhow::anyhow!("wait for result on main thread failed"))?
    }

    pub async fn show(&self) -> Result<(), anyhow::Error> {
        self.run_on_main_thread(move |win| {
            let base = win.base();
            if !base.is_visible()? {
                base.show()?;
                base.set_focus()?;
                win.restore_position()?;
            }
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn hide(&self) -> Result<(), anyhow::Error> {
        self.run_on_main_thread(move |win| {
            let base = win.base();
            if base.is_visible()? {
                win.save_position()?;
                base.hide()?;
            }
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub fn base(&self) -> WebviewWindow<R> {
        self.inner.clone()
    }
}

impl<R: tauri::Runtime> Clone for WGuiWindow<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pos: self.pos.clone(),
            ignore_menu_event: self.ignore_menu_event.clone(),
        }
    }
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
