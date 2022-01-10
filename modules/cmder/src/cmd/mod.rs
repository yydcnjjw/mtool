mod help;

use crate::{ServiceClient, ServicePoster};

use self::help::Help;

pub async fn load_buildin<CmderPoster>(cmder: ServiceClient<CmderPoster>) -> anyhow::Result<()>
where
    CmderPoster: ServicePoster + 'static,
{
    cmder.add("help".into(), Help::new(cmder.clone())).await?;
    Ok(())
}
