use async_trait::async_trait;
use futures::{FutureExt, future::LocalBoxFuture};
use minject::{InjectOnce, inject_once};

use crate::context::Context;

pub trait Callable {
    type Output;
    type Error;
    fn call(
        self: Box<Self>,
        ctx: &Context,
    ) -> LocalBoxFuture<'static, Result<Self::Output, Self::Error>>;
}

impl<Func, Args, Output> Callable for FuncWrapper<Func, (Args, Output)>
where
    Func: InjectOnce<Args, Output = Output>,
{
    type Output = Output;
    type Error = anyhow::Error;

    fn call(
        self: Box<Self>,
        ctx: &Context,
    ) -> LocalBoxFuture<'static, Result<Self::Output, Self::Error>> {
        inject_once(ctx, *self).boxed_local()
    }
}
