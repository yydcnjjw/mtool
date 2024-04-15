use std::hash::Hash;

use mapp::prelude::*;

pub struct Command<Executor> {
    label: Label,
    name: Option<String>,
    alias: Vec<String>,
    descrption: Option<String>,

    executor: Executor,
}

#[async_trait]
pub trait CommandExecutor<C> {
    async fn exec(&self, _: &C) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl<Executor, C> CommandExecutor<C> for Box<Executor>
where
    Executor: CommandExecutor<C> + ?Sized + Send + Sync,
    C: Send + Sync,
{
    async fn exec(&self, c: &C) -> Result<(), anyhow::Error> {
        self.as_ref().exec(c).await
    }
}

#[async_trait(?Send)]
pub trait LocalCommandExecutor<C> {
    async fn exec(&self, _: &C) -> Result<(), anyhow::Error>;
}

#[async_trait(?Send)]
impl<Executor, C> LocalCommandExecutor<C> for Box<Executor>
where
    Executor: LocalCommandExecutor<C> + ?Sized,
{
    async fn exec(&self, c: &C) -> Result<(), anyhow::Error> {
        self.as_ref().exec(c).await
    }
}

pub trait CommandBuilder<Executor> {
    fn label<L>(self, label: L) -> Command<Executor>
    where
        L: Into<Label>;

    fn name<T>(self, name: T) -> Command<Executor>
    where
        T: ToString;

    fn add_alias<T>(self, alias: T) -> Command<Executor>
    where
        T: ToString;

    fn descrption<T>(self, desc: T) -> Command<Executor>
    where
        T: ToString;

    fn build(self) -> Command<Executor>;
}

impl<Executor> CommandBuilder<Executor> for Command<Executor> {
    fn label<L>(mut self, label: L) -> Command<Executor>
    where
        L: Into<Label>,
    {
        self.label = label.into();
        self
    }

    fn name<T>(mut self, name: T) -> Command<Executor>
    where
        T: ToString,
    {
        self.name = Some(name.to_string());
        self
    }

    fn add_alias<T>(mut self, alias: T) -> Command<Executor>
    where
        T: ToString,
    {
        self.alias.push(alias.to_string());
        self
    }

    fn descrption<T>(mut self, desc: T) -> Command<Executor>
    where
        T: ToString,
    {
        self.descrption = Some(desc.to_string());
        self
    }

    fn build(self) -> Command<Executor> {
        self
    }
}

impl<Executor> Command<Executor>
where
    Executor: 'static,
{
    pub fn new(executor: Executor) -> Self {
        Self {
            label: Label::new::<Executor>(),
            name: None,
            alias: Vec::new(),
            descrption: None,
            executor,
        }
    }
}

impl<Executor> Command<Executor>
where
    Executor: LocalCommandExecutor<LocalInjector>,
{
    pub async fn exec_local(&self, injector: &LocalInjector) -> Result<(), anyhow::Error> {
        self.executor.exec(injector).await
    }
}

impl<Executor> Command<Executor>
where
    Executor: CommandExecutor<Injector> + Send + Sync,
{
    pub async fn exec(&self, injector: &Injector) -> Result<(), anyhow::Error> {
        self.executor.exec(injector).await
    }
}

impl<Executor> Command<Executor> {
    pub fn get_label(&self) -> &Label {
        &self.label
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref().map(|v| v.as_str()).unwrap_or_default()
    }

    pub fn get_aliases(&self) -> &Vec<String> {
        &self.alias
    }

    pub fn get_descrption(&self) -> &str {
        self.descrption
            .as_ref()
            .map(|v| v.as_str())
            .unwrap_or_default()
    }
}

impl<Executor> PartialEq for Command<Executor> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl<Executor> Hash for Command<Executor> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl<Executor> Eq for Command<Executor> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<Executor, C> From<Command<Box<Executor>>>
    for Command<Box<dyn CommandExecutor<C> + Send + Sync>>
where
    Executor: CommandExecutor<C> + Send + Sync + 'static,
{
    fn from(
        Command {
            label,
            name,
            alias,
            descrption,
            executor,
        }: Command<Box<Executor>>,
    ) -> Self {
        Self {
            label,
            name,
            alias,
            descrption,
            executor: executor as Box<dyn CommandExecutor<C> + Send + Sync>,
        }
    }
}

impl<Executor, C> From<Command<Box<Executor>>> for Command<Box<dyn LocalCommandExecutor<C>>>
where
    Executor: LocalCommandExecutor<C> + 'static,
{
    fn from(
        Command {
            label,
            name,
            alias,
            descrption,
            executor,
        }: Command<Box<Executor>>,
    ) -> Self {
        Self {
            label,
            name,
            alias,
            descrption,
            executor: executor as Box<dyn LocalCommandExecutor<C>>,
        }
    }
}
