use mapp::prelude::*;
use tauri::AppHandle;
use tokio::sync::oneshot;
use tracing::warn;

use super::window::StickyWindow;
use mtool_wgui::prelude::*;

pub(crate) fn plugin_setup<R: tauri::Runtime>(
    app: &AppHandle<R>,
    win_tx: oneshot::Sender<Res<StickyWindow<R>>>,
) -> Result<(), anyhow::Error> {
    let app = app.clone();
    tokio::spawn(async move {
        match StickyWindow::<R>::new(app).await {
            Ok(win) => {
                win.bind(win.clone());
                let _ = win_tx.send(Res::new(win));
            }
            Err(e) => warn!("{:?}", e),
        }
    });
    Ok(())
}
