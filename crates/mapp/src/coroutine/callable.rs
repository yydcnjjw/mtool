use std::future::Future;

use futures::future::LocalBoxFuture;

use crate::{
    context::Context,
    inject::{InjectOnce, LocalProvide, local_inject_once},
};

use super::wrapper::FuncWrapper;

pub trait Callable {
    type Output;
    type Error;
    fn call<'a>(
        self: Box<Self>,
        ctx: &'a Context,
    ) -> LocalBoxFuture<'a, Result<Self::Output, Self::Error>>
    where
        Self: 'a;
}

pub trait LocalInjectCallable<Args, Output, E> = InjectOnce<Args>
where
    <Self as InjectOnce<Args>>::Output: Future<Output = Result<Output, E>>,
    Context: LocalProvide<Args, Error = E>;

impl<Func, Args, Output, E> Callable for FuncWrapper<Func, (Args, Output, E)>
where
    Func: LocalInjectCallable<Args, Output, E>,
{
    type Output = Output;
    type Error = E;

    fn call<'a>(
        self: Box<Self>,
        ctx: &'a Context,
    ) -> LocalBoxFuture<'a, Result<Self::Output, Self::Error>>
    where
        Self: 'a,
    {
        Box::pin(async { local_inject_once(ctx, self.func).await? })
    }
}

pub fn new_callable<Func, Args, Output, E>(f: Func) -> Box<dyn Callable<Output = Output, Error = E>>
where
    Func: LocalInjectCallable<Args, Output, E> + 'static,
    Args: 'static,
    Output: 'static,
    E: 'static,
{
    Box::new(FuncWrapper::<Func, (Args, Output, E)>::new(f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[tokio::test]
    async fn callable() {
        let ctx = Context::new();
        ctx.provide_value(Rc::new(42i32));

        async fn my_func(v: Rc<i32>) -> Result<i32, crate::context::ContextError> {
            Ok(*v + 1)
        }

        let res = new_callable(my_func).call(&ctx).await.unwrap();

        assert_eq!(res, 43);
    }

    #[tokio::test]
    async fn custom_callable() {
        let ctx = Context::new();
        ctx.provide_value(Rc::new(42i32));

        async fn my_func(v: Rc<i32>) -> Result<i32, crate::context::ContextError> {
            Ok(*v + 1)
        }

        type CustomCallable = Box<dyn Callable<Output = i32, Error = crate::context::ContextError>>;
        let callable: CustomCallable = new_callable(my_func);
        let res = callable.call(&ctx).await.unwrap();
        assert_eq!(res, 43);
    }
}
