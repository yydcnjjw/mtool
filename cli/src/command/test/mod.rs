use std::{sync::Arc, thread};

use crate::{
    app::App,
    keybind::{KeyBinding, KeyDispatcher},
};
use sysev::{
    self,
    keydef::{KeyCode, KeyModifier},
    KeyEvent,
};

use super::Command;
use async_trait::async_trait;

struct Cmd {}

impl Cmd {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, _: Vec<String>) -> anyhow::Result<()> {
        let evbus = sysev::EventBus::new(20);

        let tx = evbus.sender();
        thread::spawn(|| {
            sysev::run_loop(tx).unwrap();
        });

        let mut dispatcher = KeyDispatcher::new(&evbus);
        dispatcher
            .bind_key(KeyBinding::new("C-c c".into(), "test".into()))
            .await?;
        dispatcher
            .bind_key(KeyBinding::new("C-c a".into(), "test".into()))
            .await?;
        dispatcher
            .bind_key(KeyBinding::new("C-S-c a".into(), "test".into()))
            .await?;

        dispatcher.run_loop().await;

        Ok(())
    }
}

pub fn add_command(app: &mut App) -> anyhow::Result<()> {
    {
        let cmd = Box::new(Cmd::new());
        app.cmder.insert("test".into(), cmd);
    }

    Ok(())
}
