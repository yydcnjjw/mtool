use std::task::Context;

use async_trait::async_trait;

#[async_trait(?Send)]
trait Runnable {
    type Error;
    async fn run(self: Box<Self>) -> Result<(), Self::Error>;
}

#[async_trait(?Send)]
impl<Func> Runnable for Func
where
    Func: FnOnce() -> Result<(), anyhow::Error>,
{
    type Error = anyhow::Error;
    async fn run(self: Box<Self>) -> Result<(), Self::Error> {
        self()
    }
}
