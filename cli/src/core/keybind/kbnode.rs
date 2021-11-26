use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use super::{kbd::KeyCombine, KeyBinding, Result};

#[derive(Debug)]
pub struct Node<Key, Value> {
    pub v: Value,
    pub childs: HashMap<Key, Arc<Mutex<Self>>>,
}

impl<Key, Value> Node<Key, Value> {
    fn new(v: Value) -> Self {
        Self {
            v,
            childs: HashMap::new(),
        }
    }
}

pub type KeyNode = Node<KeyCombine, Option<KeyBinding>>;

#[derive(Debug)]
pub struct KeyBindingRoot {
    pub root: Arc<Mutex<KeyNode>>,
}

impl KeyBindingRoot {
    pub fn new() -> Self {
        Self {
            root: Arc::new(Mutex::new(KeyNode::new(None))),
        }
    }

    pub async fn add_kcs(&mut self, kcs: Vec<KeyCombine>, kb: KeyBinding) -> Result<()> {
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

    #[allow(dead_code)]
    pub async fn remove_kcs(&mut self, _kcs: Vec<KeyCombine>, _kb: KeyBinding) -> Result<()> {
        todo!()
    }
}
