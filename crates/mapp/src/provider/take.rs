use std::{any::type_name, fmt, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use minject::Provide;

use crate::{provider::Injector, App};

pub struct Take<T>(Arc<T>);

impl<T> Take<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
    }

    pub fn take(self) -> Result<T, anyhow::Error> {
        Arc::try_unwrap(self.0).map_err(|_| anyhow::anyhow!(format!("take {}", type_name::<T>())))
    }
}

impl<T> Clone for Take<T> {
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
        app.injector()
            .remove::<Take<T>>()
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}

#[async_trait]
impl<T> Provide<Injector> for Take<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        c.remove::<Take<T>>()
            .context(format!("Failed to provide {}", type_name::<Self>()))
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
        Ok(TakeOpt(app.injector().remove::<Take<T>>()))
    }
}

#[async_trait]
impl<T> Provide<Injector> for TakeOpt<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        Ok(TakeOpt(c.remove::<Take<T>>()))
    }
}
