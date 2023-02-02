use minject_macro::{enum_params, repeat};

pub trait Inject<Args> {
    type Output;
    fn inject(&self, args: Args) -> Self::Output;
}

macro_rules! impl_inject_for_fn {
    ($($arg: ident),*) =>  {
        impl <Func, Output, $($arg,)*> Inject<($($arg,)*)> for Func
        where
            Func: Fn($($arg),*) -> Output,
        {
            type Output = Output;

            #[allow(non_snake_case)]
            fn inject(&self, ($($arg,)*): ($($arg,)*)) -> Self::Output {
                (self)($($arg,)*)
            }
        }
    }
}

repeat!(9, enum_params, impl_inject_for_fn, Arg);

pub trait InjectOnce<Args> {
    type Output;
    fn inject_once(self, args: Args) -> Self::Output;
}

macro_rules! impl_inject_once_for_fn_once {
    ($($arg: ident),*) =>  {
        impl <Func, Output, $($arg,)*> InjectOnce<($($arg,)*)> for Func
        where
            Func: FnOnce($($arg),*) -> Output,
        {
            type Output = Output;

            #[allow(non_snake_case)]
            fn inject_once(self, ($($arg,)*): ($($arg,)*)) -> Self::Output {
                (self)($($arg,)*)
            }
        }
    }
}

repeat!(9, enum_params, impl_inject_once_for_fn_once, Arg);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_inject() {
        assert!(|| -> bool { true }.inject(()));
        assert!(|_: i32| -> bool { true }.inject((1i32,)));
        assert!(|_: String| -> bool { true }.inject((String::new(),)));
        assert!(|_: Arc<i32>| -> bool { true }.inject((Arc::new(1i32),)));
    }

    #[test]
    fn test_inject_once() {
        assert!(|| -> bool { true }.inject_once(()));
        assert!(|_: i32| -> bool { true }.inject_once((1i32,)));
        assert!(|_: String| -> bool { true }.inject_once((String::new(),)));
        assert!(|_: Arc<i32>| -> bool { true }.inject_once((Arc::new(1i32),)));
    }

    #[tokio::test]
    async fn test_inject_with_async_fn() {
        assert!((|| async move { true }).inject(()).await);
        assert!((|_: i32| async move { true }).inject((1i32,)).await);
        assert!(
            (|_: String| async move { true })
                .inject((String::new(),))
                .await
        );
        assert!(
            (|_: Arc<i32>| async move { true })
                .inject((Arc::new(1i32),))
                .await
        );
    }

    #[tokio::test]
    async fn test_inject_once_with_async_fn_once() {
        assert!((|| async move { true }).inject_once(()).await);
        assert!((|_: i32| async move { true }).inject_once((1i32,)).await);
        assert!(
            (|_: String| async move { true })
                .inject_once((String::new(),))
                .await
        );
        assert!(
            (|_: Arc<i32>| async move { true })
                .inject_once((Arc::new(1i32),))
                .await
        );
    }
}
