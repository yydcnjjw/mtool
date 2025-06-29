use std::sync::Arc;

use crate::{Command, SharedCommand};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct CommandItem {
    pub title: String,
    pub sub_title: String,
    pub icon: Option<String>,
    pub hints: Vec<String>,
    pub source: String,
    pub command: SharedCommand,
    pub more_commands: Vec<SharedCommand>,
}

impl From<SharedCommand> for CommandItem {
    fn from(value: SharedCommand) -> CommandItem {
        CommandItem {
            title: value.name.clone(),
            sub_title: value.description.clone().unwrap_or_default(),
            icon: None,
            hints: Vec::new(),
            source: String::new(),
            command: value,
            more_commands: Vec::new(),
        }
    }
}

impl From<Command> for CommandItem {
    fn from(value: Command) -> CommandItem {
        CommandItem {
            title: value.name.clone(),
            sub_title: value.description.clone().unwrap_or_default(),
            icon: None,
            hints: Vec::new(),
            source: String::new(),
            command: Arc::new(value),
            more_commands: Vec::new(),
        }
    }
}
