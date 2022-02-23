mod cmd;

pub use cmd::Cmd;

use cmder_mod::ServiceClient as CmderCli;

pub async fn load(cmder: CmderCli) -> anyhow::Result<()> {
    cmder.add("toast".into(), Cmd::new()).await?;
    Ok(())
}
