use futures::{
    future::{BoxFuture, LocalBoxFuture},
    FutureExt,
};
use minject_macro::{enum_params, repeat};

// #[async_trait]
// pub trait Provide<C>: Sized {
//     async fn provide(c: &C) -> Result<Self, anyhow::Error>;
// }

// macro_rules! impl_provider_for_tuple_with_container {
//     ($($param: ident),*) => {
//         #[async_trait]
//         impl<C, $($param,)*> Provide<C> for ($($param,)*)
//         where
//             $($param: Provide<C> + Send,)*
//             C: Send + Sync + 'static,
//         {
//             #[allow(unused_variables)]
//             async fn provide(c: &C) -> Result<Self, anyhow::Error> {
//                 Ok(($($param::provide(c).await?,)*))
//             }
//         }
//     };
// }

// repeat!(9, enum_params, impl_provider_for_tuple_with_container, P);

// #[async_trait(?Send)]
// pub trait LocalProvide<C>: Sized {
//     async fn local_provide(c: &C) -> Result<Self, anyhow::Error>;
// }

// macro_rules! impl_local_provider_for_tuple_with_container {
//     ($($param: ident),*) => {
//         #[async_trait(?Send)]
//         impl<C, $($param,)*> LocalProvide<C> for ($($param,)*)
//         where
//             $($param: LocalProvide<C>,)*
//         {
//             #[allow(unused_variables)]
//             async fn local_provide(c: &C) -> Result<Self, anyhow::Error> {
//                 Ok(($($param::local_provide(c).await?,)*))
//             }
//         }
//     };
// }

// repeat!(
//     9,
//     enum_params,
//     impl_local_provider_for_tuple_with_container,
//     P
// );

pub trait LocalProvide<E, O> {
    fn local_provide(&'_ self) -> LocalBoxFuture<'_, Result<O, E>>;
}

// pub trait Provide<O, E> {
//     fn provide(&'_ self) -> BoxFuture<'_, Result<O, E>>;
// }

macro_rules! impl_local_provider_for_tuple {
    ($($param: ident),*) => {
        impl<C, E, $($param,)*> LocalProvide<E, ($($param,)*)> for C
        where
        $(C: LocalProvide<E, $param>,)*
        {
            #[allow(unused_variables)]
            fn local_provide(&'_ self) -> LocalBoxFuture<'_, Result<($($param,)*), E>> {
                async {
                    Ok(($(LocalProvide::<E, $param>::local_provide(self).await?,)*))
                }.boxed_local()
            }
        }
    };
}

repeat!(9, enum_params, impl_local_provider_for_tuple, P);

#[cfg(test)]
mod tests {
    use std::future::Future;

    use futures::{future::LocalBoxFuture, FutureExt};

    use crate::InjectOnce;

    use super::LocalProvide;

    struct Container {}

    type BoxError = Box<dyn std::error::Error>;

    impl Container {
        pub async fn local_inject_once<Func, Args, Output>(
            &self,
            f: Func,
        ) -> Result<Output, BoxError>
        where
            Func: InjectOnce<Args>,
            Func::Output: Future<Output = Output>,
            Self: LocalProvide<BoxError, Args>,
        {
            Ok(InjectOnce::<Args>::inject_once(
                f,
                LocalProvide::<BoxError, Args>::local_provide(self).await?,
            )
            .await)
        }
    }

    struct Res<T>(T);

    impl<T> LocalProvide<Box<dyn std::error::Error>, Res<T>> for Container
    where
        T: Default,
    {
        fn local_provide(
            &'_ self,
        ) -> LocalBoxFuture<'_, Result<Res<T>, Box<dyn std::error::Error>>> {
            async { Ok(Res(T::default())) }.boxed_local()
        }
    }

    #[tokio::test]
    async fn test_local_provider() {
        let c = Container {};

        c.local_inject_once(|a: Res<i32>, b: Res<i32>| async move {})
            .await
            .unwrap();

        c.local_inject_once(|a: Res<i32>| async move {})
            .await
            .unwrap();
    }
}
