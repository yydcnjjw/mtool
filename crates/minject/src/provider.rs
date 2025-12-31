use futures::future::{BoxFuture, LocalBoxFuture};

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
    ($err: path, $($param: ident),*) => {
        impl<C, $($param,)*> LocalProvide<($($param,)*)> for C
        where
        $(C: LocalProvide<$param, Error = $err>,)*
        {
            type Error = $err;
            #[allow(unused_variables)]
            fn local_provide(&'_ self) -> LocalBoxFuture<'_, Result<($($param,)*), Self::Error>> {
                Box::pin(async {
                    Ok(($(LocalProvide::<$param>::local_provide(self.c).await?,)*))
                })
            }
        }
    };
}

#[macro_export]
macro_rules! impl_local_provider {
    ($err: path) => {
        $crate::repeat!(
            9,
            $crate::enum_params,
            impl_local_provider_for_tuple,
            ($err),
            (P)
        );
    };
}

#[macro_export]
macro_rules! impl_provider_for_tuple {
    ($err: path, $($param: ident),*) => {
        impl<C, $($param,)*> Provide<($($param,)*)> for C
        where
        $($param: Send + Sync,)*
        $(C: Provide<$param, Error = $err> + Send + Sync,)*
        {
            type Error = $err;
            #[allow(unused_variables)]
            fn provide(&'_ self) -> BoxFuture<'_, Result<($($param,)*), Self::Error>> {
                Box::pin(async {
                    Ok(($(Provide::<$param>::provide(self.c).await?,)*))
                })
            }
        }
    };
}

#[macro_export]
macro_rules! impl_provider {
    ($err: path) => {
        $crate::repeat!(9, $crate::enum_params, impl_provider_for_tuple, ($err), (P));
    };
}
