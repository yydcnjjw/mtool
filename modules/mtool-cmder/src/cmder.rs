use std::{ops::Deref, rc::Rc, sync::Arc};

use dashmap::DashSet;
use mapp::prelude::*;

use crate::CommandBuilder;

type SharedCommandExecutor = Box<dyn crate::CommandExecutor<Injector> + Send + Sync>;
pub type SharedCommand = crate::Command<SharedCommandExecutor>;
pub type SharedCommandPtr = Arc<SharedCommand>;
type LocalCommandExecutor = Box<dyn crate::LocalCommandExecutor<LocalInjector>>;
pub type LocalCommand = crate::Command<LocalCommandExecutor>;
pub type LocalCommandPtr = Rc<LocalCommand>;

pub struct Cmder {
    storage: DashSet<SharedCommandPtr>,
}

impl Cmder {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self::new()))
    }

    pub fn new() -> Self {
        Self {
            storage: DashSet::new(),
        }
    }

    pub fn add_command<T, Executor>(&self, cmd: T) -> &Self
    where
        T: CommandBuilder<Box<Executor>> + Send + Sync + 'static,
        Executor: crate::CommandExecutor<Injector> + Send + Sync + 'static,
    {
        let cmd = Arc::new(SharedCommand::from(cmd.build()));

        self.storage.insert(cmd);

        self
    }

    pub fn get_command_with_alias<T>(&self, alias: T) -> Option<SharedCommandPtr>
    where
        T: AsRef<String>,
    {
        self.storage
            .iter()
            .find(|cmd| cmd.get_aliases().contains(alias.as_ref()))
            .map(|c| c.clone())
    }

    pub fn get_command_with_name(&self, name: &str) -> Option<SharedCommandPtr> {
        self.storage
            .iter()
            .find(|cmd| cmd.get_name() == name)
            .map(|c| c.clone())
    }

    pub fn get_command_with_name_or_alias(&self, name_or_alias: &str) -> Option<SharedCommandPtr> {
        let name_or_alias = name_or_alias.to_string();
        self.storage
            .iter()
            .find(|cmd| {
                cmd.get_name() == name_or_alias || cmd.get_aliases().contains(&name_or_alias)
            })
            .map(|c| c.clone())
    }

    pub fn get_command_with_label<L>(&self, label: L) -> Option<SharedCommandPtr>
    where
        L: Into<Label>,
    {
        let label = label.into();
        self.storage
            .iter()
            .find(|cmd| cmd.get_label() == &label)
            .map(|c| c.clone())
    }
}

impl Deref for Cmder {
    type Target = DashSet<SharedCommandPtr>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

pub struct LocalCmder {
    storage: DashSet<LocalCommandPtr>,
}

impl LocalCmder {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self::new()))
    }

    pub fn new() -> Self {
        Self {
            storage: DashSet::new(),
        }
    }

    pub fn add_command<T, Executor>(&self, cmd: T) -> &Self
    where
        T: CommandBuilder<Box<Executor>> + 'static,
        Executor: crate::LocalCommandExecutor<LocalInjector> + 'static,
    {
        let cmd = Rc::new(LocalCommand::from(cmd.build()));

        self.storage.insert(cmd);

        self
    }

    pub fn get_command_with_alias<T>(&self, alias: T) -> Option<LocalCommandPtr>
    where
        T: AsRef<String>,
    {
        self.storage
            .iter()
            .find(|cmd| cmd.get_aliases().contains(alias.as_ref()))
            .map(|c| c.clone())
    }

    pub fn get_command_with_name(&self, name: &str) -> Option<LocalCommandPtr> {
        self.storage
            .iter()
            .find(|cmd| cmd.get_name() == name)
            .map(|c| c.clone())
    }

    pub fn get_command_with_label<L>(&self, label: L) -> Option<LocalCommandPtr>
    where
        L: Into<Label>,
    {
        let label = label.into();
        self.storage
            .iter()
            .find(|cmd| cmd.get_label() == &label)
            .map(|c| c.clone())
    }
}

impl Deref for LocalCmder {
    type Target = DashSet<LocalCommandPtr>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use crate::CommandBuilder;

    use super::*;

    async fn test_command() -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn test_get_command() {
        let cmder = Cmder::new();

        cmder
            .add_command(test_command.name("test").add_alias("t").add_alias("te"))
            .add_command(test_command.name("aebc").add_alias("a"));

        assert_eq!(
            cmder
                .get_command_with_name("test")
                .map(|cmd| cmd.get_name()),
            Some("test")
        );
    }
}
