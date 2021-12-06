use crate::app::{App, QuitApp};

use super::evbus::{post, Event, Sender};

struct SysevQuit {}
impl SysevQuit {
    pub fn post(tx: &Sender) {
        if let Err(e) = post(tx, SysevQuit {}) {
            log::error!("{}", e);
        }
    }
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = app.evbus.sender();

    tokio::task::spawn_blocking(move || {
        let tx = sender.clone();
        if let Err(e) = sysev::run_loop(move |e| {
            if let Err(e) = post(&tx, e) {
                log::warn!("{}", e);
            }
        }) {
            log::error!("{}", e);
        }

        log::debug!("sysev run loop quit!");
        SysevQuit::post(&sender);
    });

    let mut rx = app.evbus.subscribe();
    tokio::spawn(async move {
        while let Ok(e) = rx.recv().await {
            if let Some(_) = e.downcast_ref::<Event<QuitApp>>() {
                if let Err(e) = sysev::quit() {
                    log::error!("{}", e);
                }
            } else if let Some(_) = e.downcast_ref::<Event<SysevQuit>>() {
                log::debug!("sysev quit event!");
                break;
            }
        }
    });

    Ok(())
}
