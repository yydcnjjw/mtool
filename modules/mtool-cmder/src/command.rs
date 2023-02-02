use std::{future::Future, hash::Hash, marker::PhantomData};

use async_trait::async_trait;
use mapp::{
    inject::{inject, Inject, Provide},
    provider::Injector,
    Label,
};

#[async_trait]
pub trait Command {
    async fn exec(&self, app: &Injector) -> Result<(), anyhow::Error>;
}

pub trait IntoCommand<Args> {
    type Command: Command;
    fn into_command(self) -> Self::Command;
}

pub struct FnCommand<Func, Args> {
    f: Func,
    phantom: PhantomData<Args>,
}

impl<Func, Args> FnCommand<Func, Args> {
    pub fn new(f: Func) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Func, Args, Output> Command for FnCommand<Func, Args>
where
    Func: Inject<Args, Output = Output> + Send + Sync,
    Args: Provide<Injector> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    async fn exec(&self, c: &Injector) -> Result<(), anyhow::Error> {
        inject(c, &self.f).await?.await
    }
}

impl<Func, Args, Output> IntoCommand<Args> for Func
where
    Func: Inject<Args, Output = Output> + Send + Sync,
    Args: Provide<Injector> + Send + Sync,
    Output: Future<Output = Result<(), anyhow::Error>> + Send,
{
    type Command = FnCommand<Func, Args>;

    fn into_command(self) -> Self::Command {
        FnCommand::new(self)
    }
}

type BoxedCommand = Box<dyn Command + Send + Sync>;

pub struct CommandDescriptor {
    label: Label,
    name: Option<String>,
    alias: Vec<String>,
    desc: Option<String>,
    cmd: BoxedCommand,
}

impl PartialEq for CommandDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Hash for CommandDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl Eq for CommandDescriptor {
    fn assert_receiver_is_total_eq(&self) {}
}

impl CommandDescriptor {
    pub async fn exec(&self, c: &Injector) -> Result<(), anyhow::Error> {
        self.cmd.exec(c).await
    }

    pub fn get_label(&self) -> &Label {
        &self.label
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref().map(|v| v.as_str()).unwrap_or_default()
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn get_aliases(&self) -> &Vec<String> {
        &self.alias
    }

    pub fn add_alias(mut self, alias: &str) -> Self {
        self.alias.push(alias.into());
        self
    }

    pub fn get_desc(&self) -> &str {
        self.desc.as_ref().map(|v| v.as_str()).unwrap_or_default()
    }

    pub fn set_desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.into());
        self
    }
}

fn fn_command_default<Func, Args, Output>(f: Func) -> CommandDescriptor
where
    Func: Inject<Args, Output = Output> + Send + Sync + 'static,
    Args: Provide<Injector> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    CommandDescriptor {
        label: Label::new::<Func>(),
        name: None,
        alias: Vec::new(),
        desc: None,
        cmd: Box::new(f.into_command()),
    }
}

pub trait IntoCommandDescriptor<Args> {
    fn into_command_descriptor(self) -> CommandDescriptor;
}

impl<Func, Args, Output> IntoCommandDescriptor<Args> for Func
where
    Func: Inject<Args, Output = Output>
        + IntoCommand<Args, Command = FnCommand<Func, Args>>
        + Send
        + Sync
        + 'static,
    Args: Provide<Injector> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    fn into_command_descriptor(self) -> CommandDescriptor {
        fn_command_default::<Func, Args, Output>(self)
    }
}

impl IntoCommandDescriptor<()> for CommandDescriptor {
    fn into_command_descriptor(self) -> CommandDescriptor {
        self
    }
}

pub trait CreateCommandDescriptor<Args> {
    fn label<L>(self, label: L) -> CommandDescriptor
    where
        L: Into<Label>;

    fn name(self, name: &str) -> CommandDescriptor;
}

impl<Func, Args, Output> CreateCommandDescriptor<Args> for Func
where
    Func: Inject<Args, Output = Output>
        + IntoCommand<Args, Command = FnCommand<Func, Args>>
        + Send
        + Sync
        + 'static,
    Args: Provide<Injector> + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    fn label<L>(self, label: L) -> CommandDescriptor
    where
        L: Into<Label>,
    {
        let mut cmd = fn_command_default::<Func, Args, Output>(self);
        cmd.label = label.into();
        cmd
    }

    fn name(self, name: &str) -> CommandDescriptor {
        let mut cmd = fn_command_default::<Func, Args, Output>(self);
        cmd.name = Some(name.into());
        cmd
    }
}
