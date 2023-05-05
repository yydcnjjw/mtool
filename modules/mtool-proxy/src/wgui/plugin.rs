use mapp::provider::Res;
use mtool_proxy_model::{Stats, TransferStats};
use tauri::{
    async_runtime::spawn,
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, WindowBuilder, WindowUrl, Wry,
};
use tracing::debug;

use crate::proxy::ProxyApp;

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

pub(crate) fn init(proxy_app: Res<ProxyApp>) -> TauriPlugin<Wry> {
    Builder::new("proxy")
        .setup(move |app, _| {
            let app = app.clone();
            app.manage(proxy_app);
            spawn(async move {
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
                    .expect("create mtool-proxy window");
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![stats])
        .build()
}
