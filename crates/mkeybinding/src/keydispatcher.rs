use std::{cmp, fmt};

use mapp::{
    tokio::sync::broadcast,
    tracing::{debug, warn},
};

use crate::{
    kbd::{CombineKey, KeySequence},
    keymap::KeyMap,
};

pub struct KeyDispatcher<KeyMapId, Value> {
    keymap_stack: Vec<(KeyMapId, KeyMap<Value>)>,
    current_keyseq: KeySequence,
    sender: broadcast::Sender<(KeySequence, Value)>,
}

impl<KeyMapId, Value> KeyDispatcher<KeyMapId, Value>
where
    Value: Clone,
    KeyMapId: fmt::Display + Clone + cmp::PartialEq,
{
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);

        Self {
            keymap_stack: Vec::new(),
            current_keyseq: KeySequence::empty(),
            sender: tx,
        }
    }

    pub fn push_keymap(&mut self, id: &KeyMapId, km: KeyMap<Value>) -> bool {
        if self.contains_keymap(id) {
            return false;
        }
        self.keymap_stack.push((id.clone(), km));
        true
    }

    pub fn pop_keymap(&mut self) -> Option<(KeyMapId, KeyMap<Value>)> {
        self.keymap_stack.pop()
    }

    pub fn remove_keymap(&mut self, id: &KeyMapId) -> Option<(KeyMapId, KeyMap<Value>)> {
        self.keymap_stack
            .iter()
            .position(|(key, _)| key == id)
            .map(|i| self.keymap_stack.remove(i))
    }

    pub fn contains_keymap(&self, id: &KeyMapId) -> bool {
        self.keymap_stack.iter().position(|v| &v.0 == id).is_some()
    }

    pub fn get_keymap_mut(&mut self, id: &KeyMapId) -> Option<&mut KeyMap<Value>> {
        self.keymap_stack
            .iter_mut()
            .find_map(|(key, value)| (key == id).then_some(value))
    }

    pub fn dispatch(&mut self, key: CombineKey) -> bool {
        debug!("receive key: {}", key);

        self.current_keyseq.push(key);

        for (id, km) in self.keymap_stack.iter().rev() {
            if let Ok(v) = km.lookup(&self.current_keyseq) {
                debug!("dispatch {} {}", id, self.current_keyseq.to_string());

                if let Err(e) = self.sender.send((self.current_keyseq.clone(), v.clone())) {
                    warn!("{}", e);
                }

                self.current_keyseq.clear();
                return true;
            }
        }

        self.current_keyseq.clear();

        return false;
    }

    pub fn subscribe(&self) -> broadcast::Receiver<(KeySequence, Value)> {
        self.sender.subscribe()
    }
}
