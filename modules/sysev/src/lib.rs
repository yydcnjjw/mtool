use tokio::sync::broadcast;
use msysev::Event as SysEvent;

#[mrpc::service(message(serde, debug))]
pub trait Service {
    #[rpc(message(serde(skip)))]
    fn subscribe() -> broadcast::Receiver<SysEvent>;
}

#[cfg(feature = "service")]
mod sysev;

#[cfg(feature = "service")]
pub use sysev::*;
