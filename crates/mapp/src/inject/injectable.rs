use variadics_please::all_tuples;

pub trait Inject<Args> {
    type Output;
    fn inject(&self, args: Args) -> Self::Output;
}

macro_rules! impl_inject_in_fn {
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

all_tuples!(impl_inject_in_fn, 0, 15, Arg);

pub trait InjectOnce<Args> {
    type Output;
    fn inject_once(self, args: Args) -> Self::Output;
}

macro_rules! impl_inject_once_in_fn_once {
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

all_tuples!(impl_inject_once_in_fn_once, 0, 15, Arg);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn inject() {
        assert!(|| -> bool { true }.inject(()));
        assert!(|_: i32| -> bool { true }.inject((1i32,)));
        assert!(|_: String, _: i32| -> bool { true }.inject((String::new(), 0)));
        assert!(|_: Arc<i32>| -> bool { true }.inject((Arc::new(1i32),)));
    }

    #[test]
    fn inject_once() {
        assert!(|| -> bool { true }.inject_once(()));
        assert!(|_: i32| -> bool { true }.inject_once((1i32,)));
        assert!(|_: String, _: i32| -> bool { true }.inject_once((String::new(), 0)));
        assert!(|_: Arc<i32>| -> bool { true }.inject_once((Arc::new(1i32),)));
    }

    #[test]
    fn inject_boxed() {
        assert!(Box::new(|| -> bool { true }).inject_once(()));
    }

    #[tokio::test]
    async fn async_inject() {
        assert!((|| async move { true }).inject(()).await);
        assert!((|_: i32| async move { true }).inject((1i32,)).await);
        assert!(
            (|_: String, _: i32| async move { true })
                .inject((String::new(), 0))
                .await
        );
        assert!(
            (|_: Arc<i32>| async move { true })
                .inject((Arc::new(1i32),))
                .await
        );
    }

    #[tokio::test]
    async fn async_inject_once() {
        assert!((|| async move { true }).inject_once(()).await);
        assert!((|_: i32| async move { true }).inject_once((1i32,)).await);
        assert!(
            (|_: String, _: i32| async move { true })
                .inject_once((String::new(), 0))
                .await
        );
        assert!(
            (|_: Arc<i32>| async move { true })
                .inject_once((Arc::new(1i32),))
                .await
        );
    }
}
