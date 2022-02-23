#[cfg(feature = "service")]
mod cmd;
#[cfg(feature = "service")]
mod cmder;

mod command;

pub use command::*;

#[mrpc::service(message(serde, debug))]
pub trait Service {
    #[rpc(message(serde(skip)))]
    async fn add(name: String, cmd: AsyncCommand);
    async fn remove(name: String);
    async fn list() -> Vec<String>;
    async fn exec(name: String, args: Vec<String>);
}

pub(crate) type CmderCli = ServiceClient;

#[cfg(feature = "service")]
pub async fn load(cmder: CmderCli) -> anyhow::Result<()> {
    cmd::load_buildin(cmder).await?;
    Ok(())
}

#[cfg(feature = "service")]
pub use cmder::Cmder;
