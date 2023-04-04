use tokio::sync::broadcast;
use tracing::{warn, trace};

use crate::{
    kbd::{KeyCombine, KeySequence},
    keymap::KeyMap,
};

pub struct KeyDispatcher<Value> {
    km_vec: Vec<(String, KeyMap<Value>)>,
    keyseq: KeySequence,
    tx: broadcast::Sender<Value>,
}

impl<Value> KeyDispatcher<Value>
where
    Value: Clone,
{
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);

        Self {
            km_vec: Vec::new(),
            keyseq: KeySequence::new(),
            tx,
        }
    }

    pub fn push_keymap(&mut self, id: &str, km: KeyMap<Value>) {
        self.km_vec.push((id.to_string(), km))
    }

    pub fn pop_keymap(&mut self) -> Option<(String, KeyMap<Value>)> {
        self.km_vec.pop()
    }

    pub fn remove_keymap(&mut self, id: &str) -> Option<(String, KeyMap<Value>)> {
        self.km_vec
            .iter()
            .position(|v| v.0 == id)
            .map(|i| self.km_vec.remove(i))
    }

    pub fn contains_keymap(&self, id: &str) -> bool {
        self.km_vec.iter().position(|v| v.0 == id).is_some()
    }

    pub fn get_keymap_mut(&mut self, id: &str) -> Option<&mut KeyMap<Value>> {
        self.km_vec
            .iter_mut()
            .find_map(|v| (v.0 == id).then_some(&mut v.1))
    }

    pub fn dispatch(&mut self, key: KeyCombine) -> bool {
        trace!("receive key: {}", key);

        self.keyseq.push(key);

        for (id, km) in self.km_vec.iter().rev() {
            if let Ok(v) = km.lookup(&self.keyseq) {
                trace!("dispatch {} {}", id, self.keyseq.to_string());

                if let Err(e) = self.tx.send(v.clone()) {
                    warn!("{}", e);
                }

                self.keyseq.clear();
                return true;
            }
        }

        self.keyseq.clear();

        return false;
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use crate::kbd::ToKeySequence;

    use super::*;

    fn send_key_sequence(dispatcher: &mut KeyDispatcher<i32>, kseq: &str) {
        let kseq = kseq.to_key_sequence().unwrap();

        for key in kseq.iter() {
            dispatcher.dispatch(key.clone());
        }
    }

    #[tokio::test]
    async fn test() {
        let dispatcher = Arc::new(RwLock::new(KeyDispatcher::<i32>::new()));

        let mut rx = dispatcher.read().await.subscribe();

        let dispatcher = dispatcher.clone();

        tokio::spawn(async move {
            let mut dispatcher = dispatcher.write().await;

            {
                let mut km = KeyMap::new();
                km.add("C-a a", 0).unwrap();
                km.add("C-a b", 1).unwrap();
                dispatcher.push_keymap("test", km);
            }

            send_key_sequence(&mut dispatcher, "C-a a");
            send_key_sequence(&mut dispatcher, "C-a b");

            send_key_sequence(&mut dispatcher, "C-a c");

            send_key_sequence(&mut dispatcher, "C-a a");
        });

        assert_eq!(rx.recv().await.unwrap(), 0);
        assert_eq!(rx.recv().await.unwrap(), 1);
        assert_eq!(rx.recv().await.unwrap(), 0);
    }
}
