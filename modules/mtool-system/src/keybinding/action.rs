use async_trait::async_trait;
use mapp::{
    inject::{inject, Inject, Provide},
    provider::Injector,
};
use std::{future::Future, marker::PhantomData, sync::Arc};

#[async_trait]
pub trait Action<C> {
    async fn do_action(&self, c: &C) -> Result<(), anyhow::Error>;
}

pub struct FnAction<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnAction<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, C> Action<C> for FnAction<Func, Args>
where
    Func: Inject<Args> + Send + Sync,
    Func::Output: Future<Output = Result<(), anyhow::Error>> + Send,
    Args: Provide<C> + Send + Sync,
    C: Send + Sync,
{
    async fn do_action(&self, c: &C) -> Result<(), anyhow::Error> {
        inject(c, &self.f).await?.await
    }
}

pub type SharedAction = Arc<dyn Action<Injector> + Send + Sync>;
