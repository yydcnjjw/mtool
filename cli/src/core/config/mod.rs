use crate::app::{App, QuitApp};

use self::configer::Configer;

pub mod configer;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let rx = app.evbus.subscribe();
    let tx = app.evbus.sender();
    tokio::spawn(async move {
        log::debug!("configer run loop!");
        if let Err(e) = Configer::run_loop(rx).await {
            log::error!("{}", e);
            QuitApp::post(&tx, 0);
        }
        log::debug!("configer run loop quit!");
    });
    Ok(())
}
