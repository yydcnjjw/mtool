use futures::future::{BoxFuture, LocalBoxFuture};

use crate::{
    context::{Context, ContextError},
    coroutine::Runnable,
};

type BoxTaskRunnable = Box<dyn Runnable<Error = ContextError>>;

pub struct Task {
    runnable: BoxTaskRunnable,
}

impl Task {
    fn new(runnable: BoxTaskRunnable) -> Self {
        Self { runnable }
    }

    pub fn run(self, ctx: &Context) -> LocalBoxFuture<Result<(), ContextError>> {
        self.runnable.run(ctx)
    }
}
