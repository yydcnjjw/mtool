// use std::{collections::HashMap, slice::SliceIndex};

// use crate::kbd::KeyCombine;

// trait NodeKey = Hash + std::cmp::Eq + Clone;

// #[derive(Debug)]
// struct Node<Key: NodeKey, Value> {
//     v: Option<Value>,
//     childs: HashMap<Key, Node<Key, Value>>,
// }

// impl<Key: NodeKey, Value> Node<Key, Value> {
//     fn new() -> Self {
//         Self {
//             v: None,
//             childs: HashMap::new(),
//         }
//     }

//     fn insert(&mut self, v: Key) -> &mut Node<Key, Value> {
//         self.childs.entry(v).or_insert(Node::<Key, Value>::new())
//     }

//     fn get(&self, v: &Key) -> Option<&Node<Key, Value>> {
//         self.childs.get(v)
//     }
// }

// #[derive(Debug)]
// struct KeyDispatcher {
//     tree: Node<KeyCombine, fn() -> ()>,
// }

// impl KeyDispatcher {
//     fn new() -> Self {
//         Self { tree: Node::new() }
//     }

use sysev::event::KeyEvent;
use crate::kbd::KeyCombine;
//     fn add(&mut self, seq: Vec<KeyCombine>, f: fn() -> ()) {
//         let mut node = &mut self.tree;
//         for key in seq.iter() {
//             node = node.insert(key.clone());
//         }
//         node.v = Some(f);
//     }
// }

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
    use std::io::stdout;

    use super::*;
    use crossterm::{event::EnableMouseCapture, terminal::enable_raw_mode, ExecutableCommand};

    #[test]
    fn test_key_dispatcher() {
        enable_raw_mode().unwrap();
        
        loop {
            let e = event::read().unwrap();
            match e {
                event::Event::Key(e) => {
                    let kc: KeyCombine = e.into();
                    println!("{:?}", kc)
                }
                // event::Event::Mouse(_) => todo!(),
                // event::Event::Resize(_, _) => todo!(),
                _ => {}
            }
        }
    }
}
