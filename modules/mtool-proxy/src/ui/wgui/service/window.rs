use std::{ops::Deref, sync::Arc};

use mapp::provider::{Injector, Res};
use mtool_wgui::WGuiWindow;
use tauri::{
    async_runtime::spawn,
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, WindowBuilder, WindowUrl, Wry,
};

use crate::{
    service::ProxyService,
    ui::wgui::stats::{Stats, TransferStats},
};

pub struct ProxyMonitorWindow(Arc<WGuiWindow>);

impl ProxyMonitorWindow {
    async fn new(app: tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let win = WindowBuilder::new(&app, "mtool-proxy", WindowUrl::App("/proxy".into()))
            .title("mtool-proxy")
            .transparent(true)
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .visible(true)
            // TODO: disable shadow for transparent
            .shadow(false)
            .build()
            .expect("create proxy monitor window failed");
        Ok(Self(WGuiWindow::new(win, false).await?))
    }
}

impl Deref for ProxyMonitorWindow {
    type Target = WGuiWindow;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn show_window(window: Res<ProxyMonitorWindow>) -> Result<(), anyhow::Error> {
    window.show()
}

pub async fn hide_window(window: Res<ProxyMonitorWindow>) -> Result<(), anyhow::Error> {
    window.hide()
}

#[command]
async fn stats(proxy_app: State<'_, Res<ProxyService>>) -> Result<Stats, serde_error::Error> {
    let stats = proxy_app
        .stats()
        .await
        .map_err(|e| serde_error::Error::new(&*e))?;
    Ok(Stats {
        transfer: stats
            .transfer
            .into_iter()
            .map(|(k, v)| (k, TransferStats { tx: v.tx, rx: v.rx }))
            .collect(),
    })
}

pub(crate) fn init(proxy_app: Res<ProxyService>, injector: Injector) -> TauriPlugin<Wry> {
    Builder::new("proxy")
        .setup(move |app, _| {
            let app = app.clone();
            app.manage(proxy_app);
            spawn(async move { injector.insert(Res::new(ProxyMonitorWindow::new(app).await.unwrap())) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![stats])
        .build()
}
