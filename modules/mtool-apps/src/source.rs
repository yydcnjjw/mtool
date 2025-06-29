use dioxus::prelude::*;
use mapp::{
    itertools::Itertools,
    tokio::{self, sync::watch},
    tracing::warn,
};
use mtool_cmdpal::{Command, CommandItem, CommandListWatcher, CommandSource};
use pinyin::{Pinyin, ToPinyin};
use std::sync::Arc;

use crate::{
    base64_image,
    platforms::{AppsWatcher, WindowsWatcher},
    ActiveWindowCommand, WindowInfoExt,
};

#[derive(Clone)]
pub struct AppsSource {
    watcher: AppsWatcher,
}

impl AppsSource {
    pub fn new(watcher: AppsWatcher) -> Self {
        Self { watcher }
    }
}

impl CommandSource for AppsSource {
    fn name(&self) -> String {
        "apps.apps".into()
    }

    fn subscribe(&self) -> CommandListWatcher {
        let source = self.name();

        let (tx, rx) = watch::channel(
            self.watcher
                .borrow()
                .iter()
                .map(|app| {
                    let mut item = CommandItem::from(app);
                    item.source = source.clone();
                    item
                })
                .collect_vec(),
        );

        tokio::spawn({
            to_owned![self.watcher, source];
            async move {
                if let Ok(_) = watcher.changed().await {
                    _ = tx.send(
                        watcher
                            .borrow()
                            .iter()
                            .map(|app| {
                                let mut item = CommandItem::from(app);
                                item.source = source.clone();
                                item
                            })
                            .collect_vec(),
                    )
                }
            }
        });
        rx
    }
}

pub struct WindowsSource {
    watcher: WindowsWatcher,
}

impl WindowsSource {
    pub fn new(watcher: WindowsWatcher) -> Self {
        Self { watcher }
    }
}

impl CommandSource for WindowsSource {
    fn name(&self) -> String {
        "apps.windows".into()
    }

    fn subscribe(&self) -> CommandListWatcher {
        let (tx, rx) = watch::channel(Vec::new());
        let source = self.name();
        tokio::spawn({
            to_owned![self.watcher, source];
            async move {
                while let Ok(_) = watcher.changed().await {
                    let items = watcher
                        .borrow()
                        .iter()
                        .map(|win_info| CommandItem {
                            title: win_info.process.description.clone(),
                            sub_title: win_info.title.clone(),
                            icon: win_info
                                .load_icon()
                                .and_then(|icon| base64_image(icon))
                                .inspect_err(|e| warn!("load window icon: {:#}", e))
                                .ok(),
                            hints: {
                                [
                                    &win_info.process.name,
                                    &win_info.process.description,
                                    &win_info.title,
                                ]
                                .into_iter()
                                .flat_map(|v| {
                                    [
                                        v.to_owned(),
                                        v.as_str()
                                            .to_pinyin()
                                            .filter_map(|v| v.map(Pinyin::plain))
                                            .join(" "),
                                    ]
                                })
                                .filter(|v| !v.trim().is_empty())
                                .collect_vec()
                            },
                            source: source.clone(),
                            command: Arc::new(Command::new(
                                "apps.active.window",
                                ActiveWindowCommand::new(win_info.clone()),
                            )),
                            more_commands: Vec::new(),
                        })
                        .collect_vec();

                    _ = tx.send(items)
                }
            }
        });
        rx
    }
}
