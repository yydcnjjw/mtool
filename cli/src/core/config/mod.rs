use crate::app::App;

use self::configer::Configer;

pub mod configer;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let rx = app.evbus.subscribe();
    tokio::spawn(async move {
        if let Err(e) = Configer::run_loop(rx).await {
            log::error!("{}", e);
        }
        log::debug!("configer run loop quit!");
    });
    Ok(())
}
