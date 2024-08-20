mod cmd;
mod window;
mod plugin;

pub use window::*;
pub(crate) use plugin::*;

use mapp::prelude::*;
use mtool_cmder::Cmder;

pub async fn setup(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmd::setup(cmder).await?;
    Ok(())
}
