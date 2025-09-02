use mapp::{anyhow, prelude::*};

use crate::lww::LwwStore;

pub async fn sync(_: Res<LwwStore>) -> Result<(), anyhow::Error> {
    Ok(())
}
