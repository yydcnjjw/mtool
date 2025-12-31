use std::{any::type_name, fmt, sync::Arc};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use minject::{LocalProvide, Provide};

use crate::{
    app::{App, LocalApp},
    provider::Injector,
};

use super::{LocalInjector, Res};

pub struct Take<T>(T);

impl<T> Take<T> {
    pub fn new(val: T) -> Self {
        Self(val)
    }
}

impl<T> Take<Arc<T>> {
    pub fn take(self) -> Result<T, anyhow::Error> {
        Arc::try_unwrap(self.0).map_err(|_| anyhow!(format!("take {}", type_name::<T>())))
    }
}

impl<T> Take<Res<T>> {
    pub fn take(self) -> Result<T, anyhow::Error> {
        Res::try_unwrap(self.0).map_err(|_| anyhow!(format!("take {}", type_name::<T>())))
    }
}

impl<T> Clone for Take<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Take<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Take").field(&self.0).finish()
    }
}

#[async_trait]
impl<T> Provide<App> for Take<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        Provide::provide(app.injector()).await
    }
}

#[async_trait]
impl<T> Provide<Injector> for Take<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        c.remove::<T>()
            .context(format!("Failed to provide {}", type_name::<Self>()))
            .map(|v| Take::new(v))
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalApp> for Take<T>
where
    T: 'static,
{
    async fn local_provide(app: &LocalApp) -> Result<Self, anyhow::Error> {
        LocalProvide::local_provide(app.injector()).await
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalInjector> for Take<T>
where
    T: 'static,
{
    async fn local_provide(c: &LocalInjector) -> Result<Self, anyhow::Error> {
        c.remove::<T>()
            .context(format!("Failed to provide {}", type_name::<Self>()))
            .map(|v| Take::new(v))
    }
}

pub struct TakeOpt<T>(Option<Take<T>>);

impl<T> TakeOpt<T> {
    pub fn unwrap(self) -> Option<Take<T>> {
        self.0
    }
}

#[async_trait]
impl<T> Provide<App> for TakeOpt<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        Ok(TakeOpt(app.injector().remove::<T>().map(|v| Take::new(v))))
    }
}

#[async_trait]
impl<T> Provide<Injector> for TakeOpt<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        Ok(TakeOpt(c.remove::<T>().map(|v| Take::new(v))))
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalApp> for TakeOpt<T>
where
    T: 'static,
{
    async fn local_provide(app: &LocalApp) -> Result<Self, anyhow::Error> {
        Ok(TakeOpt(app.injector().remove::<T>().map(|v| Take::new(v))))
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalInjector> for TakeOpt<T>
where
    T: 'static,
{
    async fn local_provide(c: &LocalInjector) -> Result<Self, anyhow::Error> {
        Ok(TakeOpt(c.remove::<T>().map(|v| Take::new(v))))
    }
}
