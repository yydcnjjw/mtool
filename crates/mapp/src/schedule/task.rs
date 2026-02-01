use futures::future::LocalBoxFuture;

use crate::{
    context::{Context, ContextError},
    coroutine::Runnable,
};

type BoxTaskRunnable = Box<dyn Runnable<Error = ContextError>>;

pub struct ScheduleTask {
    runnable: BoxTaskRunnable,
}

impl ScheduleTask {
    pub fn new(runnable: BoxTaskRunnable) -> Self {
        Self { runnable }
    }
    
    pub fn name(&self) -> &'static str{
        self.runnable.name()
    }
    
    pub fn run(self, ctx: &Context) -> LocalBoxFuture<'_, Result<(), ContextError>> {
        self.runnable.run(ctx)
    }
}
