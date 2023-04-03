use std::{collections::HashMap, sync::Arc};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use parking_lot::RwLock;

use mapp::{inject::Provide, provider::Res, App, Label};

use super::{CommandDescriptor, IntoCommandDescriptor};

type SharedCommandDescriptor = Arc<CommandDescriptor>;
struct CmderInner {
    fuzzy_storage: HashMap<String, SharedCommandDescriptor>,
    fuzzy_matcher: SkimMatcherV2,

    kcmap: HashMap<Label, SharedCommandDescriptor>,
}

impl CmderInner {
    fn new() -> Self {
        Self {
            fuzzy_storage: HashMap::new(),
            fuzzy_matcher: SkimMatcherV2::default(),
            kcmap: HashMap::new(),
        }
    }

    fn add_command<T, Args>(&mut self, cmd: T) -> &mut Self
    where
        T: IntoCommandDescriptor<Args> + 'static,
    {
        let cmd = Arc::new(cmd.into_command_descriptor());

        self.kcmap.insert(*cmd.get_label(), cmd.clone());

        self.fuzzy_storage
            .insert(cmd.get_name().into(), cmd.clone());

        for alias in cmd.get_aliases() {
            self.fuzzy_storage.insert(alias.clone(), cmd.clone());
        }

        self
    }

    fn get_cmd_fuzzy(&self, pattern: &str) -> Vec<SharedCommandDescriptor> {
        let mut items = Vec::new();

        for (choice, cmd) in &self.fuzzy_storage {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(choice, pattern) {
                items.push((score, cmd));
            }
        }

        items.sort_by(|a, b| a.0.cmp(&b.0));

        items
            .iter()
            .map(|item| item.1.clone())
            .unique()
            .collect_vec()
    }

    fn get_cmd_exact(&self, name_or_alias: &str) -> Option<SharedCommandDescriptor> {
        self.fuzzy_storage.get(name_or_alias).map(|v| v.clone())
    }

    fn get_cmd_with_label<L>(&self, label: L) -> Option<SharedCommandDescriptor>
    where
        L: Into<Label>,
    {
        self.kcmap.get(&label.into()).map(|v| v.clone())
    }

    fn list_command(&self) -> Vec<SharedCommandDescriptor> {
        self.kcmap.iter().map(|kv| kv.1.clone()).collect_vec()
    }
}

pub struct Cmder {
    inner: RwLock<CmderInner>,
}

impl Cmder {
    pub async fn new() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {
            inner: RwLock::new(CmderInner::new()),
        }))
    }

    pub fn add_command<Cmd, Args>(&self, cmd: Cmd) -> &Self
    where
        Cmd: IntoCommandDescriptor<Args> + 'static,
        Args: Provide<App> + Send + Sync + 'static,
    {
        self.inner.write().add_command(cmd);
        self
    }

    pub fn get_command_fuzzy(&self, pattern: &str) -> Vec<SharedCommandDescriptor> {
        self.inner.read().get_cmd_fuzzy(pattern)
    }

    pub fn get_command_exact(&self, name_or_alias: &str) -> Option<SharedCommandDescriptor> {
        self.inner.read().get_cmd_exact(name_or_alias)
    }

    pub fn get_command_with_label<L>(&self, label: L) -> Option<SharedCommandDescriptor>
    where
        L: Into<Label>,
    {
        self.inner.read().get_cmd_with_label(label)
    }

    pub fn list_command(&self) -> Vec<SharedCommandDescriptor> {
        self.inner.read().list_command()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_command() -> Result<(), anyhow::Error> {
        Ok(())
    }

    #[tokio::test]
    async fn test_get_command_fuzzy() {
        let cmder = Cmder::new().await.unwrap();

        cmder
            .add_command(test_command.name("test").add_alias("t").add_alias("te"))
            .add_command(test_command.name("aebc").add_alias("a"));

        assert_eq!(cmder.get_command_fuzzy("ae").len(), 1);
    }
}
