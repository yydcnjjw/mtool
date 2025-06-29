mod cmd;
mod plugin;
mod window;

pub(crate) use plugin::*;
pub use window::*;

use mapp::{anyhow, prelude::*};
use mtool_cmder::Cmder;

pub async fn setup(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmd::setup(cmder).await?;
    Ok(())
}
