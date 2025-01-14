use dioxus::prelude::*;
use mapp::{
    itertools::Itertools,
    sync::RwLock,
    tokio::{self, sync::watch},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{CommandItem, CommandSource, SharedCommandSource};

pub struct CommandPalette {
    inner: RwLock<CommandPaletteInner>,
}

impl CommandPalette {
    pub fn new() -> Self {
        CommandPalette {
            inner: RwLock::new(CommandPaletteInner::new()),
        }
    }
}

impl CommandPalette {
    pub fn add_top_level_command(&self, cmd: CommandItem) -> &Self {
        self.inner.write().add_top_level_command(cmd);
        self
    }

    pub fn subscribe_commands_changed(&self) -> watch::Receiver<HashMap<String, Vec<CommandItem>>> {
        self.inner.read().subscribe_commands_changed()
    }

    pub fn add_source<Source>(&self, source: Source) -> &Self
    where
        Source: CommandSource + Send + Sync + 'static,
    {
        self.inner.write().add_source(Arc::new(source));
        self
    }
}

struct CommandPaletteInner {
    commands: HashSet<CommandItem>,
    sources: Vec<SharedCommandSource>,

    source_commands: watch::Sender<HashMap<String, Vec<CommandItem>>>,
}

impl CommandPaletteInner {
    fn new() -> Self {
        let (tx, _) = watch::channel(HashMap::new());
        Self {
            commands: HashSet::new(),
            sources: Vec::new(),

            source_commands: tx,
        }
    }

    pub fn add_top_level_command(&mut self, cmd: CommandItem) {
        self.commands.insert(cmd);

        let cmds = self.commands.iter().cloned().collect_vec();

        self.source_commands.send_modify(|value| {
            value
                .entry("root".to_string())
                .and_modify(|v| *v = cmds.clone())
                .or_insert_with(|| cmds.clone());
        });
    }

    pub fn subscribe_commands_changed(&self) -> watch::Receiver<HashMap<String, Vec<CommandItem>>> {
        self.source_commands.subscribe()
    }

    pub fn add_source(&mut self, source: SharedCommandSource) {
        self.sources.push(source.clone());

        tokio::spawn({
            to_owned![self.source_commands];
            async move {
                let mut rx = source.subscribe();

                source_commands.send_modify(|value| {
                    let items = rx.borrow();
                    value
                        .entry(source.name())
                        .and_modify(|v| *v = items.clone())
                        .or_insert_with(|| items.clone());
                });

                while let Ok(_) = rx.changed().await {
                    source_commands.send_modify(|value| {
                        let items = rx.borrow();
                        value
                            .entry(source.name())
                            .and_modify(|v| *v = items.clone())
                            .or_insert_with(|| items.clone());
                    });
                }
            }
        });
    }
}
