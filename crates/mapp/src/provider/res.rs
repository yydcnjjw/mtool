use std::{
    any::{type_name, Any},
    fmt,
    marker::Unsize,
    ops::{CoerceUnsized, Deref},
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use minject::{LocalProvide, Provide};

use crate::{provider::Injector, App, LocalApp};

use super::LocalInjector;

pub struct Res<T: ?Sized>(Arc<T>);

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Res<U>> for Res<T> {}

impl<T> Res<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
    }

    pub fn new_raw(val: Arc<T>) -> Self {
        Self(val)
    }
}

impl Res<dyn Any + Send + Sync> {
    pub fn downcast<T>(self) -> Result<Res<T>, anyhow::Error>
    where
        T: Send + Sync + 'static,
    {
        Ok(Res(self.0.downcast().map_err(|_| {
            anyhow::anyhow!("can not downcast to {}", type_name::<T>())
        })?))
    }
}

impl<T: ?Sized> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> PartialEq for Res<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Clone for Res<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Res<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Res").field(&self.0).finish()
    }
}

#[async_trait]
impl<T> Provide<App> for Res<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        app.injector()
            .get::<Self>()
            .await
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}

#[async_trait]
impl<T> Provide<Injector> for Res<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        c.get::<Self>()
            .await
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalApp> for Res<T>
where
    T: 'static,
{
    async fn local_provide(app: &LocalApp) -> Result<Self, anyhow::Error> {
        app.injector()
            .get::<Self>()
            .await
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalInjector> for Res<T>
where
    T: 'static,
{
    async fn local_provide(c: &LocalInjector) -> Result<Self, anyhow::Error> {
        c.get::<Self>()
            .await
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}
