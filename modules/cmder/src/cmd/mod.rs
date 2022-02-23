mod help;

use crate::CmderCli;

use self::help::Help;

pub async fn load_buildin(cmder: CmderCli) -> anyhow::Result<()> {
    cmder.add("help".into(), Help::new(cmder.clone())).await?;
    Ok(())
}
