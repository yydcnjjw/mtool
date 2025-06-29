use std::{future::Future, marker::PhantomData};

use mapp::{anyhow, prelude::*};

use crate::{Command, CommandBuilder, CommandExecutor, LocalCommandExecutor};

pub struct FnCommand<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnCommand<Func, Args> {
    pub fn new(f: Func) -> Box<Self> {
        Box::new(Self {
            f,
            phantom: PhantomData,
        })
    }
}

#[async_trait]
impl<Func, Args, Output, C> CommandExecutor<C> for FnCommand<Func, Args>
where
    C: Send + Sync,
    Func: Inject<Args, Output = Output> + Send + Sync,
    Args: Provide<C> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    async fn exec(&self, c: &C) -> Result<(), anyhow::Error> {
        inject(c, &self.f).await?.await
    }
}

#[async_trait(?Send)]
impl<Func, Args, Output, C> LocalCommandExecutor<C> for FnCommand<Func, Args>
where
    Func: Inject<Args, Output = Output>,
    Args: LocalProvide<C>,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    async fn exec(&self, c: &C) -> Result<(), anyhow::Error> {
        local_inject(c, &self.f).await?.await
    }
}

impl<Func, Args, Output> CommandBuilder<Box<FnCommand<Func, Args>>> for Func
where
    // Add restrictions to avoid conflicts
    Func: Inject<Args, Output = Output> + 'static,
    Args: 'static,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    fn label<L>(self, label: L) -> Command<Box<FnCommand<Func, Args>>>
    where
        L: Into<Label>,
    {
        Command::new(FnCommand::new(self)).label(label)
    }

    fn name<T>(self, name: T) -> Command<Box<FnCommand<Func, Args>>>
    where
        T: ToString,
    {
        Command::new(FnCommand::new(self)).name(name)
    }

    fn add_alias<T>(self, alias: T) -> Command<Box<FnCommand<Func, Args>>>
    where
        T: ToString,
    {
        Command::new(FnCommand::new(self)).add_alias(alias)
    }

    fn descrption<T>(self, desc: T) -> Command<Box<FnCommand<Func, Args>>>
    where
        T: ToString,
    {
        Command::new(FnCommand::new(self)).descrption(desc)
    }

    fn build(self) -> Command<Box<FnCommand<Func, Args>>> {
        Command::new(FnCommand::new(self))
    }
}
