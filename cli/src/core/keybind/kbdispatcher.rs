use std::{ops::Deref, rc::Rc, cell::RefCell};

use anyhow::Context;
use futures::{future, pin_mut, StreamExt};
use sysev::{KeyAction, KeyEvent};
use tokio::sync::broadcast;

use crate::core::{
    evbus::{Event, EventBus, Receiver, Sender},
    service::Service,
};

use super::{
    kbd::{parse_kbd, KeyCombine},
    kber::KeyBinding,
    kbnode::{KeyBindingRoot, KeyNode},
    Result,
};

#[derive(Debug)]
pub struct KeyBindingDispatcher {
    root: KeyBindingRoot,
    cur_node: Option<Rc<RefCell<KeyNode>>>,
}

impl KeyBindingDispatcher {
    pub fn new() -> Self {
        Self {
            root: KeyBindingRoot::new(),
            cur_node: None,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<KeyBinding> {
        self.tx.subscribe()
    }

    pub fn add_keybinding(&mut self, kb: KeyBinding) -> Result<()> {
        let kcs = parse_kbd(kb.kbd.as_str()).context("Bind key")?;
        self.root.add_kcs(kcs, kb)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_keybinging(&mut self, _kb: KeyBinding) -> Result<()> {
        // let kcs = parse_kbd(kb.kbd.as_str()).context("Bind key")?;
        // self.root.add_kcs(kcs, kb).await?;
        // Ok(())
        todo!("remove kb")
    }

    pub async fn run_loop(tx: Sender, mut rx: Receiver) {
        let s = async_stream::stream! {
            while let Ok(item) = rx.recv().await {
                yield item;
            }
        }
        .filter_map(|e| {
            if let Some(e) = e.downcast_ref::<Event<sysev::Event>>() {
                future::ready(match e.deref() {
                    sysev::Event::Key(e) => {
                        if matches!(e.action, KeyAction::Press) {
                            Some(e)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            } else {
                future::ready(None)
            }
        });

        pin_mut!(s);

        let mut kbder = KeyBindingDispatcher::new();

        while let Some(ev) = s.next().await {
            let kc = KeyCombine::from(ev.clone());
            log::debug!("{:?}", kc);

            if kbder.cur_node.is_none() {
                kbder.cur_node = Some(kbder.root.root.clone());
            }

            let cur_node = &kbder.cur_node;
            if let Some(n) = cur_node.as_ref().childs.get(&kc) {
                match n.v {
                    Some(v) => {
                        if let Err(e) = tx.send(v.clone()) {
                            log::warn!("{}", e);
                        }
                    }
                    None => kbder.cur_node = Some(n.clone()),
                }
            } else {
                kbder.cur_node = None
            };
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
