use std::sync::Arc;

use mapp::tokio::sync::watch;

use crate::CommandItem;

pub type CommandListWatcher = watch::Receiver<Vec<CommandItem>>;

pub trait CommandSource {
    fn name(&self) -> String;
    fn subscribe(&self) -> CommandListWatcher;
}

pub type SharedCommandSource = Arc<dyn CommandSource + Send + Sync>;
