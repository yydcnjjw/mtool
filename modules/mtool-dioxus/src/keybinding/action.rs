use dioxus::prelude::*;
use mapp::{anyhow, prelude::*, send_wrapper::SendWrapper};
use std::{future::Future, sync::Arc};

#[async_trait]
pub trait RunAction {
    async fn run_action(&self) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl<Func, Output> RunAction for Func
where
    Func: Fn() -> Output + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    async fn run_action(&self) -> Result<(), anyhow::Error> {
        (self)().await
    }
}

pub type SharedAction = Arc<dyn RunAction + Send + Sync + 'static>;

#[async_trait(?Send)]
pub trait LocalRunAction {
    async fn local_run_action(&self) -> Result<(), anyhow::Error>;
}

#[async_trait(?Send)]
impl<Func, Output> LocalRunAction for Func
where
    Func: Fn() -> Output,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    async fn local_run_action(&self) -> Result<(), anyhow::Error> {
        (self)().await
    }
}

#[async_trait(?Send)]
impl LocalRunAction for Callback<(), Result<(), anyhow::Error>> {
    async fn local_run_action(&self) -> Result<(), anyhow::Error> {
        self.call(())
    }
}

pub type LocalAction = Arc<SendWrapper<Box<dyn LocalRunAction + 'static>>>;

#[derive(Clone)]
pub enum Action {
    Local(LocalAction),
    Shared(SharedAction),
}
