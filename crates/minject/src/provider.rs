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

// pub trait AnyProvide<C>: Sized {
//     type Output: Future<Output = Result<Self, anyhow::Error>>;
//     fn provide(c: &C) -> Output;
// }

// macro_rules! impl_any_provider_for_tuple_with_container {
//     ($($param: ident),*) => {
//         impl<C, $($param,)*> AnyProvide<C> for ($($param,)*)
//         where
//             $($param: AnyProvide<C>,)*
//         {
//             #[allow(unused_variables)]
//             fn provide(c: &C) -> Self::Output {
//                 Ok(($($param::local_provide(c).await?,)*))
//             }
//         }
//     };
// }

// repeat!(
//     9,
//     enum_params,
//     impl_any_provider_for_tuple_with_container,
//     P
// );
