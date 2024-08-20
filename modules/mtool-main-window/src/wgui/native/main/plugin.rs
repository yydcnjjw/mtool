use anyhow::Context as _;
use mapp::prelude::*;
use mtool_cmder::Cmder;
use mtool_wgui::prelude::*;
use tauri::{command, AppHandle, Manager, State};
use tokio::sync::oneshot;
use tracing::{debug, warn};

use super::MtoolWindow;

#[command]
pub(crate) async fn exec_command(
    cmder: State<'_, Res<Cmder>>,
    injector: State<'_, Injector>,
    command: String,
) -> Result<(), serde_error::Error> {
    let cmd = cmder
        .get_command_with_name(&command)
        .context(format!("{} not found", command))
        .map_err(|e| serde_error::Error::new(&*e))?;

    debug!("execute command with wgui: {}", cmd.get_name());

    cmd.exec(&injector)
        .await
        .map_err(|e| serde_error::Error::new(&*e))?;

    Ok(())
}

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    cmder: Res<Cmder>,
    win_tx: oneshot::Sender<Res<MtoolWindow<R>>>,
) -> Result<(), anyhow::Error> {
    app.manage(cmder);

    let app = app.clone();

    tokio::spawn(async move {
        match MtoolWindow::<R>::new(app).await {
            Ok(win) => {
                win.bind(win.clone());
                let _ = win_tx.send(Res::new(win));
            }
            Err(e) => warn!("{:?}", e),
        }
    });
    Ok(())
}
