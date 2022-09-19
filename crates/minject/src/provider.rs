use std::result::Result;

use async_trait::async_trait;
use minject_macro::{enum_params, repeat};

#[async_trait]
pub trait Provide<C>: Sized {
    async fn provide(c: &C) -> Result<Self, anyhow::Error>;
}

macro_rules! impl_provider_for_tuple_with_container {
    ($($param: ident),*) => {
        #[async_trait]
        impl<C, $($param,)*> Provide<C> for ($($param,)*)
        where
            $($param: Provide<C> + Send,)*
            C: Send + Sync + 'static,
        {
            #[allow(unused_variables)]
            async fn provide(c: &C) -> Result<Self, anyhow::Error> {
                Ok(($($param::provide(c).await?,)*))
            }
        }
    };
}

repeat!(9, enum_params, impl_provider_for_tuple_with_container, P);

#[async_trait(?Send)]
pub trait LocalProvide<C>: Sized {
    async fn local_provide(c: &C) -> Result<Self, anyhow::Error>;
}

macro_rules! impl_local_provider_for_tuple_with_container {
    ($($param: ident),*) => {
        #[async_trait(?Send)]
        impl<C, $($param,)*> LocalProvide<C> for ($($param,)*)
        where
            $($param: LocalProvide<C>,)*
        {
            #[allow(unused_variables)]
            async fn local_provide(c: &C) -> Result<Self, anyhow::Error> {
                Ok(($($param::local_provide(c).await?,)*))
            }
        }
    };
}

repeat!(
    9,
    enum_params,
    impl_local_provider_for_tuple_with_container,
    P
);
