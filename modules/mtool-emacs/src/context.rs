use std::future::Future;

use emacs::Transfer;
use mapp::{anyhow, prelude::*, tokio, tracing::warn};

pub(crate) struct EmacsContext {
    pub injector: Injector,
    pub rt: tokio::runtime::Handle,
}

impl EmacsContext {
    pub fn spawn<Func, Args>(&self, f: Func)
    where
        Func: InjectOnce<Args> + Send + 'static,
        Func::Output: Future<Output = Result<(), anyhow::Error>> + Send,
        Args: Provide<Injector> + Send,
    {
        let injector = self.injector.clone();
        self.rt.spawn(async move {
            if let Err(e) = inject_once(&injector, f).await {
                warn!("{e:?}");
            }
        });
    }
}

impl Transfer for EmacsContext {}
