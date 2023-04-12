use async_trait::async_trait;
use minject::Provide;

use crate::App;

use super::Injector;

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
