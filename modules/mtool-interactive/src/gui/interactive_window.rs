use std::ops::Deref;

use anyhow::Context;
use mapp::provider::Res;
use tauri::{AppHandle, PhysicalPosition, Window, WindowBuilder, WindowUrl};

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
            .transparent(true)
            .decorations(false)
            .resizable(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
            .build()
            .context("create interactive window")?;

        Self::adjust_position(&window)?;

        Ok(Res::new(Self { inner: window }))
    }

    fn adjust_position(window: &Window) -> Result<(), tauri::Error> {
        let primary = window.primary_monitor()?.unwrap();

        let size = primary.size();

        window.set_position(PhysicalPosition::new((size.width - 720) / 2, 200))?;
        Ok(())
    }
}

pub async fn hide_window(window: Res<InteractiveWindow>) -> Result<(), anyhow::Error> {
    window.hide()?;
    Ok(())
}
