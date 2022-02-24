mod cmd;

use cmd::{Cmd, MdictConfig};
use cmder_mod::ServiceClient as CmderCli;
use config_mod::ServiceClient as ConfigCli;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    mdict: MdictConfig,
}

pub async fn load(cmder: CmderCli, cfgcli: ConfigCli) -> anyhow::Result<()> {
    let cfg: Config = cfgcli.get_value("dict".into()).await??.try_into()?;

    cmder
        .add("dict".into(), Cmd::new(cfg.mdict.clone()))
        .await?;
    Ok(())
}
