pub mod api;
mod cmd;
pub mod window;

use mapp::prelude::*;
use mtool_cmder::Cmder;

pub async fn init(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmd::init(cmder).await?;
    Ok(())
}
