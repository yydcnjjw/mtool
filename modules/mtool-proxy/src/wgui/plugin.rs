use std::ops::Deref;

use mapp::provider::{Injector, Res};
use mtool_proxy_model::{Stats, TransferStats};
use mtool_wgui::WGuiWindow;
use tauri::{
    async_runtime::spawn,
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, WindowBuilder, WindowUrl, Wry,
};

use crate::proxy::ProxyApp;

pub struct ProxyMonitorWindow(WGuiWindow);

impl ProxyMonitorWindow {
    fn new(app: tauri::AppHandle) -> Self {
        Self(WGuiWindow::new(
            WindowBuilder::new(&app, "mtool-proxy", WindowUrl::App("/proxy".into()))
                .title("mtool-proxy")
                .transparent(true)
                .decorations(false)
                .resizable(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .visible(true)
                .shadow(true)
                .build()
                .expect("create proxy monitor window failed"),
        ))
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
async fn stats(proxy_app: State<'_, Res<ProxyApp>>) -> Result<Stats, serde_error::Error> {
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

pub(crate) fn init(proxy_app: Res<ProxyApp>, injector: Injector) -> TauriPlugin<Wry> {
    Builder::new("proxy")
        .setup(move |app, _| {
            let app = app.clone();
            app.manage(proxy_app);
            spawn(async move { injector.insert(Res::new(ProxyMonitorWindow::new(app))) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![stats])
        .build()
}
