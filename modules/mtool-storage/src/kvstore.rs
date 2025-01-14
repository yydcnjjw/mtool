use mapp::{anyhow, prelude::*};
use mtool_core::ConfigStore;

pub async fn create_kvstore(cs: Res<ConfigStore>) -> Result<Res<kv::Store>, anyhow::Error> {
    let path = cs.get::<String>("storage.kvstore").await?;
    let cfg = kv::Config::new(path);
    Ok(Res::new(kv::Store::new(cfg)?))
}
