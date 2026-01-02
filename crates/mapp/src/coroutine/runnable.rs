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
