#[cfg(feature = "service")]
mod keybinding;

#[cfg(feature = "service")]
pub use keybinding::*;
use tokio::sync::broadcast;


pub type SerdeResult<T> = std::result::Result<T, serde_error::Error>;

#[mrpc::service(message(serde, debug))]
pub trait Service {
    async fn define_key_binding(kbd: String, cmd: String) -> SerdeResult<()>;
    async fn remove_key_binding(kbd: String) -> SerdeResult<()>;
    #[rpc(message(serde(skip)))]
    async fn subscribe() -> broadcast::Receiver<String>;
}
