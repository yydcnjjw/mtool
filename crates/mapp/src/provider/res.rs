use std::{
    any::{type_name, Any},
    fmt,
    marker::Unsize,
    ops::{CoerceUnsized, Deref},
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use minject::Provide;

use crate::{provider::Injector, App};

use super::InjectorInner;

pub struct Res<T: ?Sized>(Arc<T>);

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Res<U>> for Res<T> {}

impl<T> Res<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
    }

    pub fn new_raw(val: Arc<T>) -> Self {
        Self(val)
    }

    pub fn unwrap(self) -> T {
        Arc::try_unwrap(self.0)
            .map_err(|_| anyhow::anyhow!(format!("{}", type_name::<T>())))
            .unwrap()
    }
}

impl Res<dyn Any + Send + Sync> {
    pub fn downcast<T>(self) -> Result<Res<T>, Self>
    where
        T: Any + Send + Sync,
    {
        Ok(Res(self.0.downcast().unwrap()))
    }
}

impl<T> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
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

#[async_trait]
impl<T> Provide<InjectorInner> for Res<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &InjectorInner) -> Result<Self, anyhow::Error> {
        c.get::<Self>()
            .await
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}
