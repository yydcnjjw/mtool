use crate::kbd::{parse_kbd, KeyCombine};
use anyhow::Context;
use futures_util::{future, pin_mut, StreamExt};
use sysev::{self, Event, KeyAction, KeyEvent};
use tokio::sync::Mutex;

use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::Arc,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Node<Key, Value> {
    v: Value,
    childs: HashMap<Key, Arc<Mutex<Self>>>,
}

impl<Key, Value> Node<Key, Value> {
    fn new(v: Value) -> Self {
        Self {
            v,
            childs: HashMap::new(),
        }
    }
}

type KeyNode = Node<KeyCombine, Option<KeyBinding>>;

#[derive(Debug)]
struct KeyBindingRoot {
    root: Arc<Mutex<KeyNode>>,
}

impl KeyBindingRoot {
    fn new() -> Self {
        Self {
            root: Arc::new(Mutex::new(KeyNode::new(None))),
        }
    }

    async fn add_kcs(&mut self, kcs: Vec<KeyCombine>, kb: KeyBinding) -> Result<()> {
        let mut node = self.root.clone();
        for item in kcs {
            let n = node.clone();
            node = n
                .lock()
                .await
                .childs
                .entry(item)
                .or_insert(Arc::new(Mutex::new(KeyNode::new(None))))
                .clone();
        }
        node.lock().await.v = Some(kb);
        Ok(())
    }
}

#[derive(Debug, Hash, std::cmp::Eq, PartialEq)]
pub struct KeyBinding {
    pub kbd: String,
    pub cmd_name: String,
}

impl KeyBinding {
    pub fn new(kbd: String, cmd_name: String) -> Self {
        Self { kbd, cmd_name }
    }
}

#[derive(Debug)]
pub struct KeyDispatcher {
    source: sysev::Receiver,
    root: KeyBindingRoot,
    cur_node: Option<Arc<Mutex<KeyNode>>>,
}

impl KeyDispatcher {
    pub fn new(evbus: &sysev::EventBus) -> Self {
        Self {
            source: evbus.subscribe(),
            root: KeyBindingRoot::new(),
            cur_node: None,
        }
    }

    pub async fn bind_key(&mut self, kb: KeyBinding) -> Result<()> {
        let kcs = parse_kbd(kb.kbd.as_str()).context("Bind key")?;
        self.root.add_kcs(kcs, kb).await?;
        Ok(())
    }

    pub async fn run_loop(&mut self) {
        let source = &mut self.source;
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

            if self.cur_node.is_none() {
                self.cur_node = Some(self.root.root.clone());
            }

            let cur_node = self.cur_node.clone();
            match cur_node.as_ref().unwrap().lock().await.childs.get(&kc) {
                Some(n) => match &n.lock().await.v {
                    Some(v) => {
                        println!("{:?}", v);
                    }
                    None => self.cur_node = Some(n.clone()),
                },
                None => self.cur_node = None,
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

#[cfg(test)]
mod tests {
    use crate::kbd::parse_kbd;

    use super::*;

    #[test]
    fn test_key_dispatcher() {}
}
