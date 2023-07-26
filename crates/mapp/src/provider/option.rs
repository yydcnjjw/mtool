use async_trait::async_trait;
use minject::{LocalProvide, Provide};

use crate::{App, LocalApp};

use super::{Injector, LocalInjector};

#[async_trait]
impl<T> Provide<App> for Option<T>
where
    T: Send + Sync + Clone + 'static,
{
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        Ok(app.injector().get_without_construct::<T>().await)
    }
}

#[async_trait]
impl<T> Provide<Injector> for Option<T>
where
    T: Send + Sync + Clone + 'static,
{
    async fn provide(c: &Injector) -> Result<Self, anyhow::Error> {
        Ok(c.get_without_construct::<T>().await)
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalApp> for Option<T>
where
    T: Clone + 'static,
{
    async fn local_provide(app: &LocalApp) -> Result<Self, anyhow::Error> {
        Ok(app.injector().get_without_construct::<T>().await)
    }
}

#[async_trait(?Send)]
impl<T> LocalProvide<LocalInjector> for Option<T>
where
    T: Clone + 'static,
{
    async fn local_provide(c: &LocalInjector) -> Result<Self, anyhow::Error> {
        Ok(c.get_without_construct::<T>().await)
    }
}
