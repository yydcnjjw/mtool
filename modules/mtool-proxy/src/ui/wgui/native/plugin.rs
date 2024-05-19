use mapp::prelude::*;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, Wry,
};

use crate::{
    service::ProxyService,
    ui::wgui::generic::{Stats, TransferStats},
};

#[command]
async fn stats(proxy_service: State<'_, Res<ProxyService>>) -> Result<Stats, serde_error::Error> {
    let stats = proxy_service
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

pub(crate) fn init(proxy_service: Res<ProxyService>) -> TauriPlugin<Wry> {
    Builder::new("mtool-proxy")
        .setup(move |app, _| {
            app.manage(proxy_service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![stats])
        .build()
}
