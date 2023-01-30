use tokio::sync::broadcast;

use crate::{
    kbd::{KeyCombine, KeySequence},
    keymap::{Binding, KeyMap},
};

pub struct KeyDispatcher<Value> {
    km: KeyMap<Value>,
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
            km: KeyMap::<Value>::new(),
            keyseq: KeySequence::new(),
            tx,
        }
    }

    pub fn keymap(&mut self) -> &mut KeyMap<Value> {
        &mut self.km
    }

    pub fn dispatch(&mut self, key: KeyCombine) -> bool {
        log::debug!("receive {}", key);

        self.keyseq.push(key);

        if let Ok(binding) = self.km.lookup(&self.keyseq) {
            match binding {
                Binding::Value(v) => {
                    log::debug!("dispatcher {}", self.keyseq.to_string());
                    if let Err(e) = self.tx.send(v.clone()) {
                        log::warn!("{}", e);
                    }
                    self.keyseq.clear();
                    true
                }
                Binding::Map(_) => false,
            }
        } else {
            self.keyseq.clear();
            false
        }
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
                let km = dispatcher.keymap();
                km.add("C-a a", 0).unwrap();
                km.add("C-a b", 1).unwrap();
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