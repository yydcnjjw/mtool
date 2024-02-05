use std::{future::Future, marker::PhantomData};

use anyhow::Context;

use async_trait::async_trait;
use minject::{inject_once, local_inject_once, InjectOnce, LocalProvide, Provide};
use tracing::trace;

use crate::{App, CondLoad, FnCondLoad, Label, LocalApp, LocalCondLoad};

#[async_trait]
pub trait RunOnceTask {
    async fn run_once(self, app: &App) -> Result<(), anyhow::Error>;
}

#[async_trait(?Send)]
pub trait LocalRunOnceTask {
    async fn local_run_once(self, app: &LocalApp) -> Result<(), anyhow::Error>;
}

pub trait IntoOnceTask<Args> {
    type OnceTask: RunOnceTask;
    fn into_once_task(self) -> Self::OnceTask;
}

pub trait IntoLocalOnceTask<Args> {
    type LocalOnceTask: LocalRunOnceTask;
    fn into_local_once_task(self) -> Self::LocalOnceTask;
}

pub struct FnOnceTask<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnOnceTask<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output> RunOnceTask for FnOnceTask<Func, Args>
where
    Func: InjectOnce<Args, Output = Output> + Send + Sync,
    Args: Provide<App> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    async fn run_once(self, app: &App) -> Result<(), anyhow::Error> {
        inject_once(app, self.f).await?.await
    }
}

#[async_trait(?Send)]
impl<Func, Args, Output> LocalRunOnceTask for FnOnceTask<Func, Args>
where
    Func: InjectOnce<Args, Output = Output>,
    Args: LocalProvide<LocalApp>,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    async fn local_run_once(self, app: &LocalApp) -> Result<(), anyhow::Error> {
        local_inject_once(app, self.f).await?.await
    }
}

impl<Func, Args, Output> IntoOnceTask<Args> for Func
where
    Func: InjectOnce<Args, Output = Output> + Send + Sync,
    Args: Provide<App> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    type OnceTask = FnOnceTask<Func, Args>;

    fn into_once_task(self) -> Self::OnceTask {
        FnOnceTask::new(self)
    }
}

impl<Func, Args, Output> IntoLocalOnceTask<Args> for Func
where
    Func: InjectOnce<Args, Output = Output>,
    Args: LocalProvide<LocalApp>,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    type LocalOnceTask = FnOnceTask<Func, Args>;

    fn into_local_once_task(self) -> Self::LocalOnceTask {
        FnOnceTask::new(self)
    }
}

type BoxedOnceTask = Box<dyn RunOnceTask + Send + Sync>;

type BoxedCondLoad = Box<dyn CondLoad + Send + Sync>;

pub struct OnceTaskDescriptor {
    pub label: Label,
    task: BoxedOnceTask,
    cond_load: Option<BoxedCondLoad>,
}

impl OnceTaskDescriptor {
    pub async fn run_once(self, app: &App) -> Result<(), anyhow::Error> {
        let need_load = match self.cond_load {
            Some(cond) => cond.load_with_cond(app).await?,
            None => true,
        };
        if need_load {
            trace!("run once task: {}", self.label);
            self.task
                .run_once(app)
                .await
                .context(format!("running once task: {}", self.label))?;
        }
        Ok(())
    }

    pub fn cond<Func, Args, Output>(mut self, cond: Func) -> Self
    where
        Func: InjectOnce<Args, Output = Output> + Send + Sync + 'static,
        Args: Provide<App> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        self.cond_load = Some(Box::new(FnCondLoad::new(cond)));
        self
    }
}

pub trait IntoOnceTaskDescriptor<Args> {
    fn into_once_task_descriptor(self) -> OnceTaskDescriptor;
}

impl<Func, Args, Output> IntoOnceTaskDescriptor<Args> for Func
where
    Func: InjectOnce<Args, Output = Output>
        + IntoOnceTask<Args, OnceTask = FnOnceTask<Func, Args>>
        + Send
        + Sync
        + 'static,
    Args: Provide<App> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    fn into_once_task_descriptor(self) -> OnceTaskDescriptor {
        OnceTaskDescriptor {
            label: Label::new::<Func>(),
            task: Box::new(self.into_once_task()),
            cond_load: None,
        }
    }
}

impl IntoOnceTaskDescriptor<()> for OnceTaskDescriptor {
    fn into_once_task_descriptor(self) -> OnceTaskDescriptor {
        self
    }
}

pub trait CreateOnceTaskDescriptor<Args> {
    fn label<L>(self, label: L) -> OnceTaskDescriptor
    where
        L: Into<Label>;

    fn cond<Func, Args_, Output>(self, cond: Func) -> OnceTaskDescriptor
    where
        Func: InjectOnce<Args_, Output = Output> + Send + Sync + 'static,
        Args_: Provide<App> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send;
}

impl<Func, Args, Output> CreateOnceTaskDescriptor<Args> for Func
where
    Func: InjectOnce<Args, Output = Output>
        + IntoOnceTask<Args, OnceTask = FnOnceTask<Func, Args>>
        + Send
        + Sync
        + 'static,
    Args: Provide<App> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    fn label<L>(self, label: L) -> OnceTaskDescriptor
    where
        L: Into<Label>,
    {
        OnceTaskDescriptor {
            label: label.into(),
            task: Box::new(self.into_once_task()),
            cond_load: None,
        }
    }

    fn cond<CondFunc, CondArgs, CondOutput>(self, cond: CondFunc) -> OnceTaskDescriptor
    where
        CondFunc: InjectOnce<CondArgs, Output = CondOutput> + Send + Sync + 'static,
        CondArgs: Provide<App> + Send + Sync + 'static,
        CondOutput: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        OnceTaskDescriptor {
            label: Label::new::<Func>(),
            task: Box::new(self.into_once_task()),
            cond_load: Some(Box::new(FnCondLoad::new(cond))),
        }
    }
}

type BoxedLocalOnceTask = Box<dyn LocalRunOnceTask>;

type BoxedLocalCondLoad = Box<dyn LocalCondLoad>;

pub struct LocalOnceTaskDescriptor {
    pub label: Label,
    task: BoxedLocalOnceTask,
    cond_load: Option<BoxedLocalCondLoad>,
}

impl LocalOnceTaskDescriptor {
    pub async fn run_once(self, app: &LocalApp) -> Result<(), anyhow::Error> {
        let need_load = match self.cond_load {
            Some(cond) => cond.local_load_with_cond(app).await?,
            None => true,
        };
        if need_load {
            trace!("run local once task: {}", self.label);
            self.task
                .local_run_once(app)
                .await
                .context(format!("running local once task: {}", self.label))?;
        }
        Ok(())
    }

    pub fn cond<Func, Args, Output>(mut self, cond: Func) -> Self
    where
        Func: InjectOnce<Args, Output = Output> + 'static,
        Args: LocalProvide<LocalApp> + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>>,
    {
        self.cond_load = Some(Box::new(FnCondLoad::new(cond)));
        self
    }
}

pub trait IntoLocalOnceTaskDescriptor<Args> {
    fn into_local_once_task_descriptor(self) -> LocalOnceTaskDescriptor;
}

impl<Func, Args, Output> IntoLocalOnceTaskDescriptor<Args> for Func
where
    Func: InjectOnce<Args, Output = Output>
        + IntoLocalOnceTask<Args, LocalOnceTask = FnOnceTask<Func, Args>>
        + 'static,
    Args: LocalProvide<LocalApp> + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + 'static,
{
    fn into_local_once_task_descriptor(self) -> LocalOnceTaskDescriptor {
        LocalOnceTaskDescriptor {
            label: Label::new::<Func>(),
            task: Box::new(self.into_local_once_task()),
            cond_load: None,
        }
    }
}

impl IntoLocalOnceTaskDescriptor<()> for LocalOnceTaskDescriptor {
    fn into_local_once_task_descriptor(self) -> LocalOnceTaskDescriptor {
        self
    }
}

pub trait CreateLocalOnceTaskDescriptor<Args> {
    fn label<L>(self, label: L) -> LocalOnceTaskDescriptor
    where
        L: Into<Label>;

    fn cond<Func, Args_, Output>(self, cond: Func) -> LocalOnceTaskDescriptor
    where
        Func: InjectOnce<Args_, Output = Output> + 'static,
        Args_: LocalProvide<LocalApp> + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>>;
}

impl<Func, Args, Output> CreateLocalOnceTaskDescriptor<Args> for Func
where
    Func: InjectOnce<Args, Output = Output>
        + IntoLocalOnceTask<Args, LocalOnceTask = FnOnceTask<Func, Args>>
        + 'static,
    Args: LocalProvide<LocalApp> + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + 'static,
{
    fn label<L>(self, label: L) -> LocalOnceTaskDescriptor
    where
        L: Into<Label>,
    {
        LocalOnceTaskDescriptor {
            label: label.into(),
            task: Box::new(self.into_local_once_task()),
            cond_load: None,
        }
    }

    fn cond<CondFunc, CondArgs, CondOutput>(self, cond: CondFunc) -> LocalOnceTaskDescriptor
    where
        CondFunc: InjectOnce<CondArgs, Output = CondOutput> + 'static,
        CondArgs: LocalProvide<LocalApp> + 'static,
        CondOutput: Future<Output = Result<bool, anyhow::Error>>,
    {
        LocalOnceTaskDescriptor {
            label: Label::new::<Func>(),
            task: Box::new(self.into_local_once_task()),
            cond_load: Some(Box::new(FnCondLoad::new(cond))),
        }
    }
}
