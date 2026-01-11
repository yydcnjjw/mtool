use std::future::Future;

use futures::future::LocalBoxFuture;
use minject::{InjectOnce, LocalProvide, local_inject_once};

use crate::context::Context;

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

impl<Func, Args, Output, E> Callable for FuncWrapper<Func, (Args, Output)>
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<Output, E>>,
    Context: LocalProvide<Args, Error = E>,
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

pub fn new_callable<Func, Args, Output, E>(f: Func) -> impl Callable
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<Output, E>>,
    Context: LocalProvide<Args, Error = E>,
{
    FuncWrapper::new(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[tokio::test]
    async fn test_callable() {
        let ctx = Context::new();
        ctx.provide_value(Rc::new(42i32));

        async fn my_func(v: Rc<i32>) -> Result<i32, crate::context::ContextError> {
            Ok(*v + 1)
        }

        let callable: Box<dyn Callable<Output = i32, Error = crate::context::ContextError>> =
            Box::new(FuncWrapper::new(my_func));
        let res = callable.call(&ctx).await.unwrap();
        assert_eq!(res, 43);
    }
}
