use crate::app::{App, QuitApp};

use super::evbus::{post, Event};

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = app.evbus.sender();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = sysev::run_loop(move |e| {
            if let Err(e) = post(&sender, e) {
                log::warn!("{}", e);
            }
        }) {
            log::error!("{}", e);
        }

        log::debug!("sysev run loop quit!");
    });

    let mut rx = app.evbus.subscribe();
    tokio::spawn(async move {
        while let Ok(e) = rx.recv().await {
            if let Some(_) = e.downcast_ref::<Event<QuitApp>>() {
                if let Err(e) = sysev::quit() {
                    log::error!("{}", e);
                }
                break;
            }
        }
    });

    Ok(())
}
