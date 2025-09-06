use mapp::{
    anyhow,
    async_recursion::async_recursion,
    futures::future::{BoxFuture, FutureExt},
};
use std::{fmt, future::Future, sync::Arc};

#[derive(Clone)]
pub enum MediaSource {
    Vendor(Arc<dyn Fn() -> BoxFuture<'static, Result<MediaSource, anyhow::Error>> + Send + Sync>),
    Uri(String),
}

impl fmt::Debug for MediaSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vendor(_) => f.debug_tuple("Vendor").finish(),
            Self::Uri(arg0) => f.debug_tuple("Uri").field(arg0).finish(),
        }
    }
}

impl MediaSource {
    pub fn vendor<F, Fut>(f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<MediaSource, anyhow::Error>> + Send + 'static,
    {
        MediaSource::Vendor(Arc::new(move || f().boxed()))
    }

    #[allow(unused)]
    #[async_recursion]
    pub async fn resolve_uri(&self) -> Result<String, anyhow::Error> {
        match self {
            MediaSource::Vendor(f) => f().await?.resolve_uri().await,
            MediaSource::Uri(uri) => Ok(uri.clone()),
        }
    }
}
