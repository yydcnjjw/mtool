#[cfg(feature = "service")]
mod cmder;
#[cfg(feature = "service")]
mod cmd;

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

#[cfg(feature = "service")]
pub async fn load<CmderPoster>(cmder: ServiceClient<CmderPoster>) -> anyhow::Result<()>
where
    CmderPoster: ServicePoster + 'static,
{
    cmd::load_buildin(cmder).await?;
    Ok(())
}

#[cfg(feature = "service")]
pub use cmder::Cmder;
