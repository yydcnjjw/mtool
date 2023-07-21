use std::{any::Any, marker::PhantomData};

use async_trait::async_trait;
use futures::Future;
use minject::{inject, inject_once, Inject, InjectOnce, LocalProvide, Provide, local_inject_once};

pub type BoxedAny = Box<dyn Any + Send + Sync>;

pub type LocalBoxedAny = Box<dyn Any>;

#[async_trait]
pub trait Construct<C> {
    async fn construct(&self, c: &C) -> Result<BoxedAny, anyhow::Error>;
}

pub trait IntoConstructor<Args, Output, C> {
    type Constructor: Construct<C>;
    fn into_constructor(self) -> Self::Constructor;
}

#[async_trait]
pub trait ConstructOnce<C> {
    async fn construct_once(self, c: &C) -> Result<BoxedAny, anyhow::Error>;
}

pub trait IntoOnceConstructor<Args, Output, C> {
    type OnceConstructor: ConstructOnce<C>;
    fn into_once_constructor(self) -> Self::OnceConstructor;
}

#[async_trait(?Send)]
pub trait LocalConstruct<C> {
    async fn local_construct(&self, c: &C) -> Result<LocalBoxedAny, anyhow::Error>;
}

pub trait IntoLocalConstructor<Args, Output, C> {
    type LocalConstructor: LocalConstruct<C>;
    fn into_local_constructor(self) -> Self::LocalConstructor;
}

#[async_trait(?Send)]
pub trait LocalConstructOnce<C> {
    async fn local_construct_once(self, c: &C) -> Result<LocalBoxedAny, anyhow::Error>;
}

pub trait IntoLocalOnceConstructor<Args, Output, C> {
    type LocalOnceConstructor: LocalConstructOnce<C>;
    fn into_local_once_constructor(self) -> Self::LocalOnceConstructor;
}

pub struct FnWrapper<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnWrapper<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output, C> Construct<C> for FnWrapper<Func, Args>
where
    Func: Inject<Args> + Send + Sync,
    Func::Output: Future<Output = Result<Output, anyhow::Error>> + Send,
    Args: Provide<C> + Send + Sync,
    Output: Send + Sync + 'static,
    C: Send + Sync,
{
    async fn construct(&self, c: &C) -> Result<BoxedAny, anyhow::Error> {
        inject(c, &self.f)
            .await?
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
    type Constructor = FnWrapper<Func, Args>;

    fn into_constructor(self) -> Self::Constructor {
        FnWrapper::new(self)
    }
}

#[async_trait]
impl<Func, Args, Output, C> ConstructOnce<C> for FnWrapper<Func, Args>
where
    Func: InjectOnce<Args> + Send,
    Func::Output: Future<Output = Result<Output, anyhow::Error>> + Send,
    Args: Provide<C> + Send,
    Output: Send + Sync + 'static,
    C: Send + Sync,
{
    async fn construct_once(self, c: &C) -> Result<BoxedAny, anyhow::Error> {
        inject_once(c, self.f)
            .await?
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
    type OnceConstructor = FnWrapper<Func, Args>;

    fn into_once_constructor(self) -> Self::OnceConstructor {
        FnWrapper::new(self)
    }
}

#[async_trait(?Send)]
impl<Func, Args, Output, C> LocalConstruct<C> for FnWrapper<Func, Args>
where
    Func: Inject<Args>,
    Func::Output: Future<Output = Result<Output, anyhow::Error>>,
    Args: Provide<C>,
    Output: 'static,
{
    async fn local_construct(&self, c: &C) -> Result<LocalBoxedAny, anyhow::Error> {
        inject(c, &self.f)
            .await?
            .await
            .map(|v| Box::new(v) as LocalBoxedAny)
    }
}

impl<Func, Args, Output, C> IntoLocalConstructor<Args, Output, C> for Func
where
    Func: Inject<Args>,
    Func::Output: Future<Output = Result<Output, anyhow::Error>>,
    Args: Provide<C>,
    Output: 'static,
{
    type LocalConstructor = FnWrapper<Func, Args>;

    fn into_local_constructor(self) -> Self::LocalConstructor {
        FnWrapper::new(self)
    }
}

#[async_trait(?Send)]
impl<Func, Args, Output, C> LocalConstructOnce<C> for FnWrapper<Func, Args>
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<Output, anyhow::Error>>,
    Args: LocalProvide<C>,
    Output: 'static,
{
    async fn local_construct_once(self, c: &C) -> Result<LocalBoxedAny, anyhow::Error> {
        local_inject_once(c, self.f)
            .await?
            .await
            .map(|v| Box::new(v) as LocalBoxedAny)
    }
}

impl<Func, Args, Output, C> IntoLocalOnceConstructor<Args, Output, C> for Func
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<Output, anyhow::Error>>,
    Args: LocalProvide<C>,
    Output: 'static,
{
    type LocalOnceConstructor = FnWrapper<Func, Args>;

    fn into_local_once_constructor(self) -> Self::LocalOnceConstructor {
        FnWrapper::new(self)
    }
}
