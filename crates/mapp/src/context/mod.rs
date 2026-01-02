use anyhow::anyhow;
use futures::{FutureExt, future::LocalBoxFuture};
use minject::LocalProvide;
use std::any::type_name;

use crate::inject::{LocalContainer, Res};

pub struct Context {
    local_injector: LocalContainer,
}

impl<T> LocalProvide<Res<T>> for Context
where
    T: 'static,
{
    type Error = anyhow::Error;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Res<T>, Self::Error>> {
        async {
            self.local_injector
                .consume::<Res<T>>()
                .await
                .ok_or_else(|| anyhow!(type_name::<T>()))
        }
        .boxed_local()
    }
}

impl<T> LocalProvide<Take<T>> for Context
where
    T: 'static,
{
    type Error = anyhow::Error;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Res<T>, Self::Error>> {
        async {
            self.local_injector
                .consume::<Res<T>>()
                .await
                .ok_or_else(|| anyhow!(type_name::<T>()))
        }
        .boxed_local()
    }
}
