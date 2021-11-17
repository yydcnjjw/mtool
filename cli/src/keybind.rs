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

//     fn add(&mut self, seq: Vec<KeyCombine>, f: fn() -> ()) {
//         let mut node = &mut self.tree;
//         for key in seq.iter() {
//             node = node.insert(key.clone());
//         }
//         node.v = Some(f);
//     }
// }
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};

use crate::kbd::{Key, KeyCombine, KeyMods};

impl From<KeyCode> for Key {
    fn from(val: KeyCode) -> Self {
        match val {
            KeyCode::F(n) => Key::Fn(n),
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Null => Key::Null,
            KeyCode::Esc => Key::Esc,
        }
    }
}

impl From<KeyModifiers> for KeyMods {
    fn from(val: KeyModifiers) -> Self {
        let mut km = KeyMods::NONE;

        if val.contains(KeyModifiers::SHIFT) {
            km |= KeyMods::SHIFT
        }
        if val.contains(KeyModifiers::CONTROL) {
            km |= KeyMods::CTRL
        }
        if val.contains(KeyModifiers::ALT) {
            km |= KeyMods::ALT
        }
        km
    }
}

impl From<KeyEvent> for KeyCombine {
    fn from(val: KeyEvent) -> Self {
        KeyCombine {
            key: val.code.into(),
            mods: val.modifiers.into(),
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
