use std::marker::PhantomData;

use async_trait::async_trait;
use futures::Future;
use minject::{inject_once, InjectOnce, Provide};

use crate::App;

#[async_trait]
pub trait CondLoad {
    async fn load_with_cond(self, app: &App) -> Result<bool, anyhow::Error>;
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
