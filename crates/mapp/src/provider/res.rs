use std::{any::type_name, ops::Deref, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use minject::Provide;

use crate::{App, Injector};

pub struct Res<T>(Arc<T>);

impl<T> Res<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
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

#[async_trait]
impl<T> Provide<App> for Res<T>
where
    T: Send + Sync + 'static,
{
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        app.injector()
            .get::<Self>()
            .await
            .map(|data| data.clone())
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
            .map(|data| data.clone())
            .context(format!("Failed to provide {}", type_name::<Self>()))
    }
}
