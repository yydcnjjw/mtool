use std::{ops::Deref, sync::RwLock};

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
    pub fn new(window: tauri::Window) -> Self {
        Self {
            inner: window,
            pos: RwLock::new(PhysicalPosition::new(0, 0)),
        }
    }

    fn save_position(&self) -> Result<(), anyhow::Error> {
        let mut pos = self.pos.write().unwrap();
        *pos = self.inner.outer_position()?;
        info!("save position: {:?}", &pos);
        Ok(())
    }

    fn restore_position(&self) -> Result<(), anyhow::Error> {
        let pos = self.pos.read().unwrap();
        self.inner.set_position(pos.clone())?;
        info!("restore position: {:?}", &pos);
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

pub struct MtoolWindow(WGuiWindow);

impl MtoolWindow {
    fn new(app: AppHandle) -> Self {
        Self(WGuiWindow::new(
            WindowBuilder::new(&app, "mtool", WindowUrl::App("index.html".into()))
                .title("mtool")
                .transparent(true)
                .decorations(false)
                .resizable(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .visible(false)
                .build()
                .expect("create mtool window failed"),
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
    Builder::new("window")
        .setup(move |app, _| {
            let app = app.clone();
            spawn(async move {
                injector.insert(Res::new(MtoolWindow::new(app)));
            });
            Ok(())
        })
        .build()
}
