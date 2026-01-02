mod error;
mod injectable;
mod provider;

pub use injectable::{Inject, InjectOnce};
pub use minject_macro::*;
pub use provider::{LocalProvide, Provide};

use std::future::Future;

pub async fn inject<'a, Func, Args, Output, C, E>(c: &'a C, f: &Func) -> Result<Output, E>
where
    Func: Inject<Args>,
    Func::Output: Future<Output = Output>,
    C: Provide<Args, Error = E>,
{
    Ok(f.inject(c.provide().await?).await)
}

pub async fn inject_blocking<'a, Func, Args, Output, C, E>(c: &'a C, f: &Func) -> Result<Output, E>
where
    Func: Inject<Args, Output = Output>,
    C: Provide<Args, Error = E>,
{
    Ok(f.inject(c.provide().await?))
}

pub async fn inject_once<'a, Func, Args, Output, C, E>(c: &'a C, f: Func) -> Result<Output, E>
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Output>,
    C: Provide<Args, Error = E>,
{
    Ok(f.inject_once(c.provide().await?).await)
}

pub async fn inject_once_blocking<'a, Func, Args, Output, C, E>(
    c: &'a C,
    f: Func,
) -> Result<Output, E>
where
    Func: InjectOnce<Args, Output = Output>,
    C: Provide<Args, Error = E>,
{
    Ok(f.inject_once(c.provide().await?))
}

pub async fn local_inject<'a, Func, Args, Output, C, E>(c: &'a C, f: &Func) -> Result<Output, E>
where
    Func: Inject<Args>,
    Func::Output: Future<Output = Output>,
    C: LocalProvide<Args, Error = E>,
{
    Ok(f.inject(c.local_provide().await?).await)
}

pub async fn local_inject_blocking<'a, Func, Args, Output, C, E>(
    c: &'a C,
    f: &Func,
) -> Result<Output, E>
where
    Func: Inject<Args, Output = Output>,
    C: LocalProvide<Args, Error = E>,
{
    Ok(f.inject(c.local_provide().await?))
}

pub async fn local_inject_once<'a, Func, Args, Output, C, E>(c: &'a C, f: Func) -> Result<Output, E>
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Output>,
    C: LocalProvide<Args, Error = E>,
{
    Ok(f.inject_once(c.local_provide().await?).await)
}

pub async fn local_inject_once_blocking<'a, Func, Args, Output, C, E>(
    c: &'a C,
    f: Func,
) -> Result<Output, E>
where
    Func: InjectOnce<Args, Output = Output>,
    C: LocalProvide<Args, Error = E>,
{
    Ok(f.inject_once(c.local_provide().await?))
}

#[cfg(test)]
mod tests {
    use futures::future::{BoxFuture, LocalBoxFuture};

    use crate::{
        LocalProvide, Provide, inject, inject_blocking, inject_once, inject_once_blocking,
        local_inject, local_inject_blocking, local_inject_once, local_inject_once_blocking,
    };

    struct Container {}

    struct Res<T>(T);

    impl<T> LocalProvide<Res<T>> for Container
    where
        T: Default,
    {
        type Error = anyhow::Error;
        fn local_provide(&'_ self) -> LocalBoxFuture<'_, Result<Res<T>, anyhow::Error>> {
            Box::pin(async { Ok(Res(T::default())) })
        }
    }

    impl<T> Provide<Res<T>> for Container
    where
        T: Default,
    {
        type Error = anyhow::Error;
        fn provide(&'_ self) -> BoxFuture<'_, Result<Res<T>, anyhow::Error>> {
            Box::pin(async { Ok(Res(T::default())) })
        }
    }

    #[tokio::test]
    async fn provider() {
        let c = Container {};

        local_inject(&c, &|_: Res<i32>, _: Res<i32>| async move {})
            .await
            .unwrap();

        local_inject(&c, &|_: Res<i32>| async move {})
            .await
            .unwrap();

        inject(&c, &|_: Res<i32>, _: Res<i32>| async move {})
            .await
            .unwrap();

        inject(&c, &|_: Res<i32>| async move {}).await.unwrap();

        local_inject_once(&c, |_: Res<i32>, _: Res<i32>| async move {})
            .await
            .unwrap();

        local_inject_once(&c, |_: Res<i32>| async move {})
            .await
            .unwrap();

        inject_once(&c, |_: Res<i32>, _: Res<i32>| async move {})
            .await
            .unwrap();

        inject_once(&c, |_: Res<i32>| async move {}).await.unwrap();
    }

    #[tokio::test]
    async fn provider_blocking() {
        let c = Container {};

        local_inject_blocking(&c, &|_: Res<i32>, _: Res<i32>| {})
            .await
            .unwrap();

        local_inject_blocking(&c, &|_: Res<i32>| {}).await.unwrap();

        inject_blocking(&c, &|_: Res<i32>, _: Res<i32>| {})
            .await
            .unwrap();

        inject_blocking(&c, &|_: Res<i32>| {}).await.unwrap();

        local_inject_once_blocking(&c, |_: Res<i32>, _: Res<i32>| {})
            .await
            .unwrap();

        local_inject_once_blocking(&c, |_: Res<i32>| {})
            .await
            .unwrap();

        inject_once_blocking(&c, |_: Res<i32>, _: Res<i32>| {})
            .await
            .unwrap();

        inject_once_blocking(&c, |_: Res<i32>| {}).await.unwrap();
    }
}
