use std::ops::Deref;

use anyhow::Context;
use mapp::provider::Res;
use tauri::{AppHandle, PhysicalPosition, PhysicalSize, WindowBuilder, WindowUrl};

pub struct InteractiveWindow {
    inner: tauri::Window,
}

impl Deref for InteractiveWindow {
    type Target = tauri::Window;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl InteractiveWindow {
    pub async fn new(app: Res<AppHandle>) -> Result<Res<Self>, anyhow::Error> {
        Self::new_inner(app.deref().clone())
    }

    pub fn new_inner(app: AppHandle) -> Result<Res<Self>, anyhow::Error> {
        let window = WindowBuilder::new(&app, "interactive", WindowUrl::App("index.html".into()))
            .title("mtool interactive")
            .transparent(if cfg!(unix) {
                true
            } else if cfg!(windows) {
                false
            } else {
                true
            })
            .decorations(false)
            .resizable(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
            .build()
            .context("create interactive window")?;

        Ok(Res::new(Self { inner: window }))
    }

    pub fn show(&self) -> Result<(), tauri::Error> {
        let primary = self.primary_monitor()?.unwrap();

        let size = primary.size();

        self.set_position(PhysicalPosition::new((size.width - 720) / 2, 200))?;

        self.set_size(PhysicalSize::new(720, 48 + 16 * 10))?;

        self.inner.show()?;

        Ok(())
    }
}

pub async fn hide_window(window: Res<InteractiveWindow>) -> Result<(), anyhow::Error> {
    window.hide()?;
    Ok(())
}
