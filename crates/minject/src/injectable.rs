use async_trait::async_trait;
use minject_macro::{enum_params, repeat};

#[async_trait]
pub trait Inject<Args> {
    type Output;
    async fn inject(&self, args: Args) -> Self::Output;
}

macro_rules! impl_inject_for_fn {
    ($($arg: ident),*) =>  {
        #[async_trait]
        impl <Func, Output, $($arg,)*> Inject<($($arg,)*)> for Func
        where
            Func: Fn($($arg),*) -> Output + Send + Sync,
            $($arg: Send + 'static,)*
        {
            type Output = Output;

            #[allow(non_snake_case)]
            async fn inject(&self, ($($arg,)*): ($($arg,)*)) -> Self::Output {
                (self)($($arg,)*)
            }
        }
    }
}

repeat!(9, enum_params, impl_inject_for_fn, Arg);

#[async_trait(?Send)]
pub trait LocalInject<Args> {
    type Output;
    async fn local_inject(&self, args: Args) -> Self::Output;
}

macro_rules! impl_local_inject_for_fn {
    ($($arg: ident),*) =>  {
        #[async_trait(?Send)]
        impl <Func, Output, $($arg,)*> LocalInject<($($arg,)*)> for Func
        where
            Func: Fn($($arg),*) -> Output,
            $($arg: 'static,)*
        {
            type Output = Output;

            #[allow(non_snake_case)]
            async fn local_inject(&self, ($($arg,)*): ($($arg,)*)) -> Self::Output {
                (self)($($arg,)*)
            }
        }
    }
}

repeat!(9, enum_params, impl_local_inject_for_fn, Arg);

#[cfg(test)]
mod tests {
    use std::{rc::Rc, sync::Arc};

    use super::*;

    #[tokio::test]
    async fn test_inject() {
        assert!(|| -> bool { true }.inject(()).await);
        assert!(|_: i32| -> bool { true }.inject((1i32,)).await);
        assert!(|_: String| -> bool { true }.inject((String::new(),)).await);
        assert!(
            |_: Arc<i32>| -> bool { true }
                .inject((Arc::new(1i32),))
                .await
        );
    }

    #[tokio::test]
    async fn test_local_inject() {
        assert!(|| -> bool { true }.local_inject(()).await);
        assert!(|_: i32| -> bool { true }.local_inject((1i32,)).await);
        assert!(
            |_: String| -> bool { true }
                .local_inject((String::new(),))
                .await
        );
        assert!(
            |_: Arc<i32>| -> bool { true }
                .local_inject((Arc::new(1i32),))
                .await
        );
        assert!(
            |_: Rc<i32>| -> bool { true }
                .local_inject((Rc::new(1i32),))
                .await
        );
    }
}
