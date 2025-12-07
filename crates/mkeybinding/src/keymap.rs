use crate::{
    error::Error,
    kbd::{CombineKey, KeySequence},
};
use mapp::anyhow;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub enum Node<Value> {
    Value(Value),
    Map(KeyMap<Value>),
}

#[derive(Debug)]
pub struct KeyMap<Value> {
    trie: HashMap<CombineKey, Node<Value>>,
}

impl<Value> KeyMap<Value> {
    pub fn new() -> KeyMap<Value> {
        Self {
            trie: HashMap::new(),
        }
    }

    pub fn add<T>(&mut self, kseq: T, value: Value) -> Result<(), Error>
    where
        T: TryInto<KeySequence, Error = anyhow::Error>,
    {
        let kseq: KeySequence = kseq.try_into()?;
        let (last, rest) = kseq.split_last().unwrap();
        let mut km = self;
        for (i, key) in rest.iter().enumerate() {
            let entry = km
                .trie
                .entry(key.clone())
                .or_insert(Node::Map(KeyMap::<Value>::new()));

            km = match entry {
                Node::Map(v) => v,
                _ => {
                    return Err(Error::KeySequenceExisted {
                        kseq: kseq.clone(),
                        prefix: kseq[0..i].into(),
                    });
                }
            };
        }

        km.trie.insert(last.clone(), Node::Value(value));
        Ok(())
    }

    pub fn remove<T>(&mut self, kseq: T) -> Result<(), Error>
    where
        T: TryInto<KeySequence, Error = anyhow::Error>,
    {
        let kseq: KeySequence = kseq.try_into()?;
        let (last, rest) = kseq.split_last().unwrap();
        let mut km = self;
        for (i, key) in rest.iter().enumerate() {
            let binding = match km.trie.get_mut(key) {
                Some(v) => v,
                None => return Ok(()),
            };

            km = match binding {
                Node::Map(v) => v,
                Node::Value(_) => {
                    return Err(Error::KeySequenceExisted {
                        kseq: kseq.clone(),
                        prefix: kseq[0..i].into(),
                    });
                }
            };
        }

        km.trie.remove(last);

        Ok(())
    }

    pub fn lookup(&self, kseq: &KeySequence) -> Option<&Value> {
        let (last, rest) = kseq.split_last().unwrap();
        let mut km = self;

        for key in rest {
            km = match km.trie.get(key)? {
                Node::Map(v) => v,
                _ => return None,
            }
        }

        km.trie.get(last).and_then(|v| match v {
            Node::Value(v) => Some(v),
            Node::Map(_) => None,
        })
    }
}

impl<Value> Clone for KeyMap<Value>
where
    Value: Clone,
{
    fn clone(&self) -> Self {
        Self {
            trie: self.trie.clone(),
        }
    }
}
