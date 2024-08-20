mod api;
mod cmd;
mod plugin;
mod window;

pub use api::*;
pub(crate) use plugin::plugin_setup;
pub use window::*;

use mapp::prelude::*;
use mtool_cmder::Cmder;

pub(crate) async fn setup(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmd::setup(cmder).await?;
    Ok(())
}
