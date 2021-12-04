use crate::app::App;

use super::evbus::post;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = app.evbus.sender();
    tokio::task::spawn_blocking(move || {
        let sender = sender.clone();
        if let Err(e) = sysev::run_loop(move |e| {
            if let Err(e) = post(&sender, e) {
                log::warn!("{}", e);
            }
        }) {
            log::error!("{}", e);
        }
    });
    Ok(())
}
