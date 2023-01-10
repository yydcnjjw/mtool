use std::{any::type_name, future::Future, marker::PhantomData};

use anyhow::Context;

use async_trait::async_trait;
use minject::{self, Inject, Provide};

use crate::{App, Label};

#[async_trait]
pub trait Task {
    async fn run(&self, app: &App) -> Result<(), anyhow::Error>;
}

pub trait IntoTask<Args> {
    type Task: Task;
    fn into_task(self) -> Self::Task;
}

pub struct FnTask<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnTask<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output> Task for FnTask<Func, Args>
where
    Func: Inject<Args, Output = Output> + Send + Sync,
    Args: Provide<App> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    async fn run(&self, app: &App) -> Result<(), anyhow::Error> {
        Ok(self
            .f
            .inject(
                Args::provide(app)
                    .await
                    .context(format!("Failed to inject {}", type_name::<Args>()))?,
            )
            .await
            .await?)
    }
}

impl<Func, Args, Output> IntoTask<Args> for Func
where
    Func: Inject<Args, Output = Output> + Send + Sync,
    Args: Provide<App> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    type Task = FnTask<Func, Args>;

    fn into_task(self) -> Self::Task {
        FnTask::new(self)
    }
}

#[async_trait]
pub trait CondLoad {
    async fn load_with_cond(&self, app: &App) -> Result<bool, anyhow::Error>;
}

pub struct FnCondLoad<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnCondLoad<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output> CondLoad for FnCondLoad<Func, Args>
where
    Func: Inject<Args, Output = Output> + Send + Sync,
    Args: Provide<App> + Send + Sync,
    Output: Future<Output = Result<bool, anyhow::Error>> + Send,
{
    async fn load_with_cond(&self, app: &App) -> Result<bool, anyhow::Error> {
        Ok(self
            .f
            .inject(
                Args::provide(app)
                    .await
                    .context(format!("Failed to inject {}", type_name::<Args>()))?,
            )
            .await
            .await?)
    }
}

type BoxedCondLoad = Box<dyn CondLoad + Send + Sync>;

type BoxedTask = Box<dyn Task + Send + Sync>;
pub struct TaskDescriptor {
    pub label: Label,
    task: BoxedTask,
    pub cond_load: Option<BoxedCondLoad>,
}

impl TaskDescriptor {
    pub async fn run(&self, app: &App) -> Result<(), anyhow::Error> {
        self.task.run(app).await
    }

    pub fn cond<Func, Args, Output>(mut self, cond: Func) -> Self
    where
        Func: Inject<Args, Output = Output> + Send + Sync + 'static,
        Args: Provide<App> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        self.cond_load = Some(Box::new(FnCondLoad::new(cond)));
        self
    }
}

pub trait IntoTaskDescriptor<Args> {
    fn into_task_descriptor(self) -> TaskDescriptor;
}

impl<Func, Args, Output> IntoTaskDescriptor<Args> for Func
where
    Func: Inject<Args, Output = Output>
        + IntoTask<Args, Task = FnTask<Func, Args>>
        + Send
        + Sync
        + 'static,
    Args: Provide<App> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    fn into_task_descriptor(self) -> TaskDescriptor {
        TaskDescriptor {
            label: Label::new::<Func>(),
            task: Box::new(self.into_task()),
            cond_load: None,
        }
    }
}

impl IntoTaskDescriptor<()> for TaskDescriptor {
    fn into_task_descriptor(self) -> TaskDescriptor {
        self
    }
}

pub trait CreateTaskDescriptor<Args> {
    fn label<L>(self, label: L) -> TaskDescriptor
    where
        L: Into<Label>;

    fn cond<Func, Args_, Output>(self, cond: Func) -> TaskDescriptor
    where
        Func: Inject<Args_, Output = Output> + Send + Sync + 'static,
        Args_: Provide<App> + Send + Sync + 'static,
        Output: Future<Output = Result<bool, anyhow::Error>> + Send;
}

impl<Func, Args, Output> CreateTaskDescriptor<Args> for Func
where
    Func: Inject<Args, Output = Output>
        + IntoTask<Args, Task = FnTask<Func, Args>>
        + Send
        + Sync
        + 'static,
    Args: Provide<App> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    fn label<L>(self, label: L) -> TaskDescriptor
    where
        L: Into<Label>,
    {
        TaskDescriptor {
            label: label.into(),
            task: Box::new(self.into_task()),
            cond_load: None,
        }
    }

    fn cond<Func_, Args_, Output_>(self, cond: Func_) -> TaskDescriptor
    where
        Func_: Inject<Args_, Output = Output_> + Send + Sync + 'static,
        Args_: Provide<App> + Send + Sync + 'static,
        Output_: Future<Output = Result<bool, anyhow::Error>> + Send,
    {
        TaskDescriptor {
            label: Label::new::<Func>(),
            task: Box::new(self.into_task()),
            cond_load: Some(Box::new(FnCondLoad::new(cond))),
        }
    }
}
