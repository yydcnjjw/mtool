use std::{collections::HashMap, ops::Deref, sync::Arc};

use tokio::sync::RwLock;

use super::{kbd::KeyCombine, kber::KeyBinding, Result};

#[derive(Debug)]
pub struct Node<Key, Value> {
    pub v: Value,
    pub childs: HashMap<Key, SharedNode<Key, Value>>,
}

impl<Key, Value> Node<Key, Value> {
    fn new(v: Value) -> Self {
        Self {
            v,
            childs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedNode<Key, Value> {
    inner: Arc<RwLock<Node<Key, Value>>>,
}

impl<Key, Value> SharedNode<Key, Value> {
    fn new(v: Value) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Node::new(v))),
        }
    }
}

impl<Key, Value> Deref for SharedNode<Key, Value> {
    type Target = Arc<RwLock<Node<Key, Value>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// pub type KeyNode = Node<KeyCombine, Option<KeyBinding>>;
pub type SharedKeyNode = SharedNode<KeyCombine, Option<KeyBinding>>;

#[derive(Debug)]
pub struct KeyBindingRoot {
    pub root: SharedKeyNode,
}

impl KeyBindingRoot {
    pub fn new() -> Self {
        Self {
            root: SharedKeyNode::new(None),
        }
    }

    pub async fn add_kcs(&mut self, kcs: Vec<KeyCombine>, kb: KeyBinding) -> Result<()> {
        let mut node = self.root.clone();
        for item in kcs {
            let n = node.clone();
            node = n
                .write()
                .await
                .childs
                .entry(item)
                .or_insert(SharedKeyNode::new(None))
                .clone();
        }
        node.write().await.v = Some(kb);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_kcs(&mut self, _kcs: Vec<KeyCombine>, _kb: KeyBinding) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::keybind::kbd::parse_kbd;

    use super::*;

    #[tokio::test]
    async fn test_name() {
        let mut kbr = KeyBindingRoot::new();

        {
            let kcs = parse_kbd("ctrl+a b").unwrap();
            kbr.add_kcs(
                kcs,
                KeyBinding {
                    kbd: "ctrl+a b".into(),
                    cmd_name: "".into(),
                },
            )
            .await
            .unwrap();
        }

        {
            let kcs = parse_kbd("ctrl+a c").unwrap();
            kbr.add_kcs(
                kcs,
                KeyBinding {
                    kbd: "ctrl+a bc".into(),
                    cmd_name: "".into(),
                },
            )
            .await
            .unwrap();
        }
    }
}
