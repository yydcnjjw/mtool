mod cmd;
mod mdx;

use std::sync::Arc;

use cmd::DictCmd;
use cmder_mod::ServiceClient as CmderCli;
use config_mod::ServiceClient as ConfigCli;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    mdict: mdx::MdictConfig,
}

pub async fn load(cmder: CmderCli, cfgcli: ConfigCli) -> anyhow::Result<()> {
    cmder
        .add("de".into(), Arc::new(Mutex::new(DictCmd::CollinsDict)))
        .await?;
    cmder
        .add("dt".into(), Arc::new(Mutex::new(DictCmd::CollinsThsaures)))
        .await?;

    let cfg: Config = cfgcli.get_value("dict".into()).await??.try_into()?;

    cmder.add("md".into(), mdx::Cmd::new(cfg.mdict)).await?;

    Ok(())
}
