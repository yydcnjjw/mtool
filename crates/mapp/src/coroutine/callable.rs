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
