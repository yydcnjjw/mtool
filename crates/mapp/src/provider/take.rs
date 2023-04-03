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
        Arc::try_unwrap(self.0).map_err(|_| anyhow::anyhow!(format!("{}", type_name::<T>())))
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
            .remove::<Self>()
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}

#[async_trait]
impl<T> Provide<Injector> for Take<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        c.remove::<Self>()
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}
