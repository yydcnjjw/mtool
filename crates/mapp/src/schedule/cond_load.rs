use std::marker::PhantomData;

use async_trait::async_trait;
use futures::Future;
use minject::{inject_once, local_inject_once, InjectOnce, LocalProvide, Provide};

use crate::{App, LocalApp};

#[async_trait]
pub trait CondLoad {
    async fn load_with_cond(self, app: &App) -> Result<bool, anyhow::Error>;
}

#[async_trait(?Send)]
pub trait LocalCondLoad {
    async fn local_load_with_cond(self, app: &LocalApp) -> Result<bool, anyhow::Error>;
}

pub struct FnCondLoad<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnCondLoad<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output> CondLoad for FnCondLoad<Func, Args>
where
    Func: InjectOnce<Args, Output = Output> + Send + Sync,
    Args: Provide<App> + Send + Sync,
    Output: Future<Output = Result<bool, anyhow::Error>> + Send,
{
    async fn load_with_cond(self, app: &App) -> Result<bool, anyhow::Error> {
        inject_once(app, self.f).await?.await
    }
}

#[async_trait(?Send)]
impl<Func, Args, Output> LocalCondLoad for FnCondLoad<Func, Args>
where
    Func: InjectOnce<Args, Output = Output>,
    Args: LocalProvide<LocalApp>,
    Output: Future<Output = Result<bool, anyhow::Error>>,
{
    async fn local_load_with_cond(self, app: &LocalApp) -> Result<bool, anyhow::Error> {
        local_inject_once(app, self.f).await?.await
    }
}
