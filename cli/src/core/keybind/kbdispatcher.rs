use std::ops::Deref;

use anyhow::Context;
use sysev::{KeyAction, KeyEvent};

use crate::core::evbus::{post, post_result, Event, Receiver, ResponsiveEvent, Sender};

use super::{
    kbd::{parse_kbd, KeyCombine},
    kber::KeyBinding,
    kbnode::{KeyBindingRoot, SharedKeyNode},
    Result,
};

#[derive(Debug)]
pub struct KeyBindingDispatcher {
    root: KeyBindingRoot,
    cur_node: Option<SharedKeyNode>,
    tx: Sender,
}

impl KeyBindingDispatcher {
    pub fn new(tx: Sender) -> Self {
        Self {
            root: KeyBindingRoot::new(),
            cur_node: None,
            tx,
        }
    }

    pub async fn add_keybinding(&mut self, kb: KeyBinding) -> Result<()> {
        let kcs = parse_kbd(kb.kbd.as_str()).context("Bind key")?;
        self.root.add_kcs(kcs, kb).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_keybinging(&mut self, _kb: KeyBinding) -> Result<()> {
        todo!("remove kb")
    }

    async fn dispatch(&mut self, e: &KeyEvent) {
        let kc = KeyCombine::from(e.clone());

        if self.cur_node.is_none() {
            self.cur_node = Some(self.root.root.clone());
        }

        let cur_node = self.cur_node.as_ref().unwrap().clone();
        if let Some(child_node) = cur_node.read().await.childs.get(&kc) {
            match &child_node.read().await.v {
                Some(v) => {
                    if let Err(e) = post(&self.tx, v.clone()) {
                        log::warn!("{}", e);
                    }
                }
                None => self.cur_node = Some(child_node.clone()),
            }
        } else {
            self.cur_node = None
        };
    }

    pub async fn run_loop(tx: Sender, mut rx: Receiver) {
        let mut kbder = KeyBindingDispatcher::new(tx);

        while let Ok(e) = rx.recv().await {
            if let Some(e) = e.downcast_ref::<Event<sysev::Event>>() {
                match e.deref() {
                    sysev::Event::Key(e) => {
                        if matches!(e.action, KeyAction::Press) {
                            kbder.dispatch(e).await;
                        }
                    }
                    // _ => {}
                }
            } else if let Some(e) =
                e.downcast_ref::<ResponsiveEvent<DefineKeyBinding, anyhow::Result<()>>>()
            {
                e.result(
                    kbder
                        .add_keybinding(e.kb.clone())
                        .await
                        .context("Add KeyBinding"),
                );
            }
        }
    }
}

impl From<KeyEvent> for KeyCombine {
    fn from(val: KeyEvent) -> Self {
        KeyCombine {
            key: val.keycode,
            mods: val.modifiers,
        }
    }
}

pub struct DefineKeyBinding {
    kb: KeyBinding,
}

impl DefineKeyBinding {
    pub async fn post(sender: &Sender, kbd: &str, cmd: &str) -> anyhow::Result<anyhow::Result<()>> {
        post_result::<DefineKeyBinding, anyhow::Result<()>>(
            sender,
            DefineKeyBinding {
                kb: KeyBinding {
                    kbd: kbd.into(),
                    cmd_name: cmd.into(),
                },
            },
        )
        .await
    }
}
