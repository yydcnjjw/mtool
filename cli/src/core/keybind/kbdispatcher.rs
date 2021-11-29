use std::sync::Arc;

use anyhow::Context;
use futures::{future, pin_mut, StreamExt};
use sysev::{Event, KeyAction, KeyEvent};
use tokio::sync::{broadcast, Mutex};

use crate::core::evbus::{EventBus, Receiver};

use super::{
    kbd::{parse_kbd, KeyCombine},
    kbnode::{KeyBindingRoot, KeyNode},
    KeyBinding, Result,
};

#[derive(Debug)]
pub struct KeyBindingDispatcher {
    ev_source: Receiver,
    root: KeyBindingRoot,
    cur_node: Option<Arc<Mutex<KeyNode>>>,

    tx: broadcast::Sender<KeyBinding>,
}

impl KeyBindingDispatcher {
    pub fn new(evbus: &EventBus) -> Self {
        let (tx, _) = broadcast::channel(5);
        Self {
            ev_source: evbus.subscribe(),
            root: KeyBindingRoot::new(),
            cur_node: None,
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<KeyBinding> {
        self.tx.subscribe()
    }

    pub async fn add_keybinding(&mut self, kb: KeyBinding) -> Result<()> {
        let kcs = parse_kbd(kb.kbd.as_str()).context("Bind key")?;
        self.root.add_kcs(kcs, kb).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_keybinging(&mut self, _kb: KeyBinding) -> Result<()> {
        // let kcs = parse_kbd(kb.kbd.as_str()).context("Bind key")?;
        // self.root.add_kcs(kcs, kb).await?;
        // Ok(())
        todo!("remove kb")
    }

    pub async fn run_loop(&mut self) {
        let source = &mut self.ev_source;
        let s = async_stream::stream! {
            while let Ok(item) = source.recv().await {
                yield item;
            }
        }
        .filter_map(|ev| {
            future::ready(match ev {
                Event::Key(e) => {
                    if matches!(e.action, KeyAction::Press) {
                        Some(e)
                    } else {
                        None
                    }
                }
                _ => None,
            })
        });

        pin_mut!(s);

        while let Some(ev) = s.next().await {
            let kc = KeyCombine::from(ev);
            log::debug!("{:?}", kc);

            if self.cur_node.is_none() {
                self.cur_node = Some(self.root.root.clone());
            }

            let cur_node = self.cur_node.clone();
            if let Some(n) = cur_node.as_ref().unwrap().lock().await.childs.get(&kc) {
                match &n.lock().await.v {
                    Some(v) => {
                        if let Err(e) = self.tx.send(v.clone()) {
                            log::warn!("{}", e);
                        }
                    }
                    None => self.cur_node = Some(n.clone()),
                }
            } else {
                self.cur_node = None
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
