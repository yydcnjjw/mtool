mod cmd;

pub use cmd::Cmd;

use cmder_mod::ServiceClient as CmderCli;

pub async fn load<CmderPoster>(cmder: CmderCli<CmderPoster>) -> anyhow::Result<()>
where
    CmderPoster: cmder_mod::ServicePoster,
{
    cmder.add("toast".into(), Cmd::new()).await?;
    Ok(())
}
