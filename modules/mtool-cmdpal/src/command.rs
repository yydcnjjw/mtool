use dioxus::prelude::Element;
use mapp::{anyhow, prelude::*};
use std::{future::Future, hash::Hash, sync::Arc};

pub struct Command {
    pub name: String,
    pub description: Option<String>,
    inner: Box<dyn InvokeCommand + Send + Sync>,
}

pub type SharedCommand = Arc<Command>;

impl Command {
    pub fn new<Name, Invoke>(name: Name, invoke: Invoke) -> Self
    where
        Name: ToString,
        Invoke: InvokeCommand + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            description: None,
            inner: Box::new(invoke),
        }
    }

    pub fn description<T>(mut self, value: T) -> Self
    where
        T: ToString,
    {
        self.description = Some(value.to_string());
        self
    }

    pub async fn invoke(&self) -> Result<CommandResult, anyhow::Error> {
        self.inner.invoke().await
    }
}

impl Eq for Command {}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Command {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub enum CommandResult {
    ShowView(fn() -> Element),
    ShowHome,
    Dismiss,  // Reset the palette to the main view and dismiss
    Hide,     // Keep this view open, but hide the palette.
    DoNothing, // Do nothing.
}

#[async_trait]
pub trait InvokeCommand {
    async fn invoke(&self) -> Result<CommandResult, anyhow::Error>;
}

#[async_trait]
impl<T, O> InvokeCommand for T
where
    T: Fn() -> O + Send + Sync,
    O: Future<Output = Result<CommandResult, anyhow::Error>> + Send,
{
    async fn invoke(&self) -> Result<CommandResult, anyhow::Error> {
        self().await
    }
}
