use std::path::PathBuf;

use mapp::{anyhow, prelude::*};
use mtool_core::ConfigStore;

pub async fn create_kvstore(cs: Res<ConfigStore>) -> Result<Res<kv::Store>, anyhow::Error> {
    let path = cs
        .get_optional::<PathBuf>("storage.kvstore")
        .unwrap_or(cs.root_path().await.join("cache/kvcache"));
    let cfg = kv::Config::new(path);
    Ok(Res::new(kv::Store::new(cfg)?))
}
