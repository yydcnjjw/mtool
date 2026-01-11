use std::future::Future;

use futures::future::LocalBoxFuture;
use minject::{InjectOnce, LocalProvide, local_inject_once};

use crate::context::Context;

use super::wrapper::FuncWrapper;

pub trait Runnable {
    type Error;
    fn run<'a>(self: Box<Self>, ctx: &'a Context) -> LocalBoxFuture<'a, Result<(), Self::Error>>
    where
        Self: 'a;
}

impl<Func, Args, E> Runnable for FuncWrapper<Func, Args>
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<(), E>>,
    Context: LocalProvide<Args, Error = E>,
{
    type Error = E;

    fn run<'a>(self: Box<Self>, ctx: &'a Context) -> LocalBoxFuture<'a, Result<(), Self::Error>>
    where
        Self: 'a,
    {
        Box::pin(async { local_inject_once(ctx, self.func).await? })
    }
}

pub fn new_runnable<Func, Args, E>(f: Func) -> impl Runnable
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<(), E>>,
    Context: LocalProvide<Args, Error = E>,
{
    FuncWrapper::new(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[tokio::test]
    async fn test_runnable() {
        let ctx = Context::new();
        ctx.provide_value(Rc::new(42i32));

        async fn my_func(v: Rc<i32>) -> Result<(), crate::context::ContextError> {
            assert_eq!(*v, 42);
            Ok(())
        }

        let runnable = Box::new(FuncWrapper::new(my_func));
        runnable.run(&ctx).await.unwrap();
    }
}
