use tokio::io::{AsyncRead, AsyncWrite};

pub trait AsyncIO: AsyncRead + AsyncWrite {}

impl<T> AsyncIO for T where T: AsyncRead + AsyncWrite {}

pub type BoxedAsyncIO = Box<dyn AsyncIO + Send + Unpin>;
