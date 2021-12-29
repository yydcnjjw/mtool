mod cmd;
mod language;
mod tencent;
mod translator;

use cmd::Cmd;
use cmder_mod::ServiceClient as CmderCli;
use config_mod::ServiceClient as ConfigCli;
use language::LanguageType;
use tencent::Config;

pub async fn load<CmderPoster, ConfigPoster>(
    cmder: CmderCli<CmderPoster>,
    cfgcli: ConfigCli<ConfigPoster>,
) -> anyhow::Result<()>
where
    CmderPoster: cmder_mod::ServicePoster,
    ConfigPoster: config_mod::ServicePoster,
{
    let cfg: Config = cfgcli.get_value("translate".into()).await??.try_into()?;

    cmder
        .add(
            "tz".into(),
            Cmd::new(cfg.clone(), LanguageType::Auto, LanguageType::Zh),
        )
        .await?;
    cmder
        .add(
            "te".into(),
            Cmd::new(cfg.clone(), LanguageType::Auto, LanguageType::En),
        )
        .await?;
    Ok(())
}
