use futures::{FutureExt, future::LocalBoxFuture};
use minject::{ContainerWrapper, InjectOnce, LocalProvide, local_inject_once};

use crate::context::Context;

use super::wrapper::FuncWrapper;

pub trait Runnable {
    type Error;
    fn run<'a>(self: Box<Self>, ctx: &'a Context) -> LocalBoxFuture<'a, Result<(), Self::Error>>
    where
        Self: 'a;
}

impl<Func, Args> Runnable for FuncWrapper<Func, Args>
where
    Func: InjectOnce<Args>,
    Func::Output: Future<Output = Result<(), anyhow::Error>>,
    for<'a> ContainerWrapper<'a, Context, anyhow::Error>: LocalProvide<Args, Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn run<'a>(self: Box<Self>, ctx: &'a Context) -> LocalBoxFuture<'a, Result<(), Self::Error>>
    where
        Self: 'a,
    {
        async { local_inject_once(ctx, (*self).func).await? }.boxed_local()
    }
}
