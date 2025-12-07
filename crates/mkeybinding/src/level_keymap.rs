use std::{
    collections::{HashMap, HashSet},
    hash::{self, Hash},
};

use mapp::petgraph::{prelude::*, visit::NodeIndexable};

use crate::KeyMap;

// struct NamedKeyMap<Value> {
//     name: String,
//     activatable: Box<dyn Fn() -> bool>,
//     keymap: KeyMap<Value>,
// }

// struct LevelKeyMap<Value> {
//     keymap_alist: HashMap<String, NamedKeyMap<Value>>,
//     graph: UnGraph<String, ()>,
//     root: String,
// }

// impl<Value> LevelKeyMap<Value> {
//     pub fn set_root(&mut self, keymap: NamedKeyMap<Value>) {
//         let name = keymap.name.clone();
//         self.add_keymap(keymap);
//         self.root = name;
//     }

//     pub fn insert_keymap(&mut self, parent: String, keymap: NamedKeyMap<Value>) {
//         let name = keymap.name.clone();
//         let index = self.add_keymap(keymap);
//         self.graph.add_edge(parent, index, ());
//     }

//     fn add_keymap(&mut self, keymap: NamedKeyMap<Value>) -> NodeIndex {
//         let name = keymap.name.clone();
//         self.keymap_alist.insert(name.clone(), keymap);
//         self.graph.add_node(name)
//     }
// }

// window1 window2
// modal  window1.view

// define_view_keymap
// define_modal_keymap
// define_window_keymap
