use std::{ops::Deref, sync::RwLock};

use anyhow::Context;
use mapp::provider::{Injector, Res};
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    AppHandle, PhysicalPosition, WindowBuilder, WindowUrl, Wry,
};
use tracing::info;

pub struct WGuiWindow {
    inner: tauri::Window,
    pos: RwLock<PhysicalPosition<i32>>,
}

impl Deref for WGuiWindow {
    type Target = tauri::Window;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl WGuiWindow {
    fn new(app: &AppHandle) -> Result<Self, anyhow::Error> {
        let window = WindowBuilder::new(app, "mtool", WindowUrl::App("index.html".into()))
            .title("mtool")
            .transparent(true)
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
            .build()
            .context("create window")?;

        Ok(Self {
            inner: window,
            pos: RwLock::new(PhysicalPosition::new(0, 0)),
        })
    }

    fn save_position(&self) -> Result<(), anyhow::Error> {
        *self.pos.write().unwrap() = self.inner.outer_position()?;

        info!("save position: {:?}", self.pos);
        Ok(())
    }

    fn restore_position(&self) -> Result<(), anyhow::Error> {
        let pos = self.pos.read().unwrap();
        self.inner.set_position(pos.clone())?;

        info!("restore position: {:?}", pos);
        Ok(())
    }

    pub fn show(&self) -> Result<(), anyhow::Error> {
        self.inner.show()?;
        self.restore_position()?;
        Ok(())
    }

    pub fn hide(&self) -> Result<(), anyhow::Error> {
        self.save_position()?;
        self.inner.hide()?;
        Ok(())
    }
}

#[allow(dead_code)]
pub(crate) async fn show_window(window: Res<WGuiWindow>) -> Result<(), anyhow::Error> {
    window.show()
}

pub(crate) async fn hide_window(window: Res<WGuiWindow>) -> Result<(), anyhow::Error> {
    window.hide()?;
    Ok(())
}

pub(crate) fn init(injector: Injector) -> TauriPlugin<Wry> {
    Builder::new("window")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move {
                injector.insert(Res::new(WGuiWindow::new(&app).unwrap()));
            });

            Ok(())
        })
        .build()
}
