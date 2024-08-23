mod api;
mod cmd;
mod window;
mod wry_plugin;

pub use api::*;
pub use window::*;
pub use wry_plugin::*;

use mapp::prelude::*;
use mtool_cmder::Cmder;

pub(crate) async fn setup<R>(cmder: Res<Cmder>, injector: Injector) -> Result<(), anyhow::Error>
where
    R: tauri::Runtime,
{
    cmd::setup(cmder).await?;
    injector.construct_once(StickyWindow::<R>::construct);
    Ok(())
}
