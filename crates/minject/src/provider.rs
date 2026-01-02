use futures::future::{BoxFuture, LocalBoxFuture};
use minject_macro::{enum_params, repeat};

pub trait LocalProvide<O> {
    type Error;
    fn local_provide(&'_ self) -> LocalBoxFuture<'_, Result<O, Self::Error>>;
}

pub trait Provide<O> {
    type Error;
    fn provide(&'_ self) -> BoxFuture<'_, Result<O, Self::Error>>;
}

#[macro_export]
macro_rules! impl_local_provider_for_tuple {
    ($($param: ident),*) => {
        impl<C, E, $($param,)*> LocalProvide<($($param,)*)> for C
        where
        $(C: LocalProvide<$param, Error = E>,)*
        {
            type Error = E;
            #[allow(unused_variables)]
            fn local_provide(&'_ self) -> LocalBoxFuture<'_, Result<($($param,)*), Self::Error>> {
                Box::pin(async {
                    Ok(($(LocalProvide::<$param>::local_provide(self).await?,)*))
                })
            }
        }
    };
}

repeat!(1, 9, enum_params, impl_local_provider_for_tuple, P);

macro_rules! impl_provider_for_tuple {
    ($($param: ident),*) => {
        impl<C, E, $($param,)*> Provide<($($param,)*)> for C
        where
        E: Send + Sync,
        $($param: Send + Sync,)*
        $(C: Provide<$param, Error = E> + Send + Sync,)*
        {
            type Error = E;
            #[allow(unused_variables)]
            fn provide(&'_ self) -> BoxFuture<'_, Result<($($param,)*), Self::Error>> {
                Box::pin(async {
                    Ok(($(Provide::<$param>::provide(self).await?,)*))
                })
            }
        }
    };
}

repeat!(1, 9, enum_params, impl_provider_for_tuple, P);
