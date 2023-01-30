use std::{
    any::{type_name, Any},
    marker::PhantomData,
};

use anyhow::Context;
use async_trait::async_trait;
use futures::Future;
use minject::{Inject, InjectOnce, Provide};

pub type BoxedAny = Box<dyn Any + Send + Sync>;

#[async_trait]
pub trait Construct<C> {
    async fn construct(&self, c: &C) -> Result<BoxedAny, anyhow::Error>;
}

pub trait IntoConstructor<Args, Output, C> {
    type Constructor: Construct<C>;
    fn into_constructor(self) -> Self::Constructor;
}

pub struct FnConstructor<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnConstructor<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output, C> Construct<C> for FnConstructor<Func, Args>
where
    Func: Inject<Args> + Send + Sync,
    Func::Output: Future<Output = Result<Output, anyhow::Error>> + Send,
    Args: Provide<C> + Send + Sync,
    Output: Send + Sync + 'static,
    C: Send + Sync,
{
    async fn construct(&self, c: &C) -> Result<BoxedAny, anyhow::Error> {
        self.f
            .inject(
                Args::provide(c)
                    .await
                    .context(format!("Failed to inject {}", type_name::<Args>()))?,
            )
            .await
            .await
            .map(|v| Box::new(v) as BoxedAny)
    }
}

impl<Func, Args, Output, C> IntoConstructor<Args, Output, C> for Func
where
    Func: Inject<Args> + Send + Sync,
    Func::Output: Future<Output = Result<Output, anyhow::Error>> + Send,
    Args: Provide<C> + Send + Sync,
    Output: Send + Sync + 'static,
    C: Send + Sync,
{
    type Constructor = FnConstructor<Func, Args>;

    fn into_constructor(self) -> Self::Constructor {
        FnConstructor::new(self)
    }
}

#[async_trait]
pub trait ConstructOnce<C> {
    async fn construct_once(self, c: &C) -> Result<BoxedAny, anyhow::Error>;
}

pub trait IntoOnceConstructor<Args, Output, C> {
    type OnceConstructor: ConstructOnce<C>;
    fn into_once_constructor(self) -> Self::OnceConstructor;
}

pub struct FnOnceConstructor<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnOnceConstructor<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output, C> ConstructOnce<C> for FnOnceConstructor<Func, Args>
where
    Func: InjectOnce<Args> + Send,
    Func::Output: Future<Output = Result<Output, anyhow::Error>> + Send,
    Args: Provide<C> + Send,
    Output: Send + Sync + 'static,
    C: Send + Sync,
{
    async fn construct_once(self, c: &C) -> Result<BoxedAny, anyhow::Error> {
        self.f
            .inject_once(
                Args::provide(c)
                    .await
                    .context(format!("Failed to inject {}", type_name::<Args>()))?,
            )
            .await
            .await
            .map(|v| Box::new(v) as BoxedAny)
    }
}

impl<Func, Args, Output, C> IntoOnceConstructor<Args, Output, C> for Func
where
    Func: InjectOnce<Args> + Send,
    Func::Output: Future<Output = Result<Output, anyhow::Error>> + Send,
    Args: Provide<C> + Send,
    Output: Send + Sync + 'static,
    C: Send + Sync,
{
    type OnceConstructor = FnOnceConstructor<Func, Args>;

    fn into_once_constructor(self) -> Self::OnceConstructor {
        FnOnceConstructor::new(self)
    }
}
