use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{kbd::KeyCombine, kber::KeyBinding, Result};

#[derive(Debug)]
pub struct Node<Key, Value> {
    pub v: Value,
    pub childs: HashMap<Key, Rc<RefCell<Self>>>,
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
    pub root: Rc<RefCell<KeyNode>>,
}

impl KeyBindingRoot {
    pub fn new() -> Self {
        Self {
            root: Rc::new(RefCell::new(KeyNode::new(None))),
        }
    }

    pub fn add_kcs(&mut self, kcs: Vec<KeyCombine>, kb: KeyBinding) -> Result<()> {
        let mut node = self.root.clone();
        for item in kcs {
            let n = node.clone();
            node = n
                .borrow_mut()
                .childs
                .entry(item)
                .or_insert(Rc::new(RefCell::new(KeyNode::new(None))))
                .clone();
        }
        node.borrow_mut().v = Some(kb);
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

    #[test]
    fn test_name() {
        let mut kbr = KeyBindingRoot::new();

        {
            let kcs = parse_kbd("ctrl+a b").unwrap();
            kbr.add_kcs(
                kcs,
                KeyBinding {
                    kbd: "ctrl+a b",
                    cmd_name: "".into(),
                },
            );
        }

        {
            let kcs = parse_kbd("ctrl+a c").unwrap();
            kbr.add_kcs(
                kcs,
                KeyBinding {
                    kbd: "ctrl+a bc",
                    cmd_name: "".into(),
                },
            );
        }
    }
}
