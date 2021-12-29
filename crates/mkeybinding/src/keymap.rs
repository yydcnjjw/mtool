use std::{collections::HashMap, fmt::Debug};

use anyhow::Context;

use crate::kbd::{self, KeyCombine, KeySequence, ToKeySequence};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Key sequence not found {0}")]
    KeySequenceNotFound(KeySequence),
    #[error("Key sequence {key} starts with non-prefix key {prefix}")]
    KeySequenceExisted {
        key: KeySequence,
        prefix: KeySequence,
    },
    #[error("{0}")]
    Kbd(#[from] kbd::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Binding<Value> {
    Value(Value),
    Map(KeyMap<Value>),
}

#[derive(Debug)]
pub struct KeyMap<Value> {
    inner: HashMap<KeyCombine, Binding<Value>>,
}

impl<Value> KeyMap<Value> {
    pub fn new() -> KeyMap<Value> {
        Self {
            inner: HashMap::new(),
        }
    }

    fn parse_key_sequence<T>(kseq: T) -> Result<KeySequence>
    where
        T: ToKeySequence,
    {
        let kseq: KeySequence = kseq
            .to_key_sequence()
            .context("Failed to parse key sequence")?;
        assert!(!kseq.is_empty());
        Ok(kseq)
    }

    fn add_binding<T>(&mut self, kseq: T, v: Binding<Value>) -> Result<()>
    where
        T: ToKeySequence,
    {
        let kseq = Self::parse_key_sequence(kseq)?;

        let (last, rest) = kseq.split_last().unwrap();
        let mut km = self;
        for (i, key) in rest.iter().enumerate() {
            let entry = km
                .inner
                .entry(key.clone())
                .or_insert(Binding::Map(KeyMap::<Value>::new()));

            km = match entry {
                Binding::Map(v) => v,
                _ => {
                    return Err(Error::KeySequenceExisted {
                        key: kseq.clone(),
                        prefix: kseq[0..i].to_key_sequence()?,
                    });
                }
            };
        }

        km.inner.insert(last.clone(), v);
        Ok(())
    }

    pub fn add<T>(&mut self, kseq: T, v: Value) -> Result<()>
    where
        T: ToKeySequence,
    {
        self.add_binding(kseq, Binding::Value(v))
    }

    pub fn remove<T>(&mut self, kseq: T) -> Result<()>
    where
        T: ToKeySequence,
    {
        let kseq = Self::parse_key_sequence(kseq)?;

        let (last, rest) = kseq.split_last().unwrap();
        let mut km = self;
        for (i, key) in rest.iter().enumerate() {
            let binding = match km.inner.get_mut(key) {
                Some(v) => v,
                None => return Ok(()),
            };

            km = match binding {
                Binding::Map(v) => v,
                Binding::Value(_) => {
                    return Err(Error::KeySequenceExisted {
                        key: kseq.clone(),
                        prefix: kseq[0..i].to_key_sequence()?,
                    });
                }
            };
        }

        km.inner.remove(last);

        Ok(())
    }

    pub fn lookup<T>(&self, kseq: T) -> Result<&Binding<Value>>
    where
        T: ToKeySequence,
    {
        let kseq = Self::parse_key_sequence(kseq)?;

        let (last, rest) = kseq.split_last().unwrap();
        let mut km = self;

        for key in rest {
            km = if let Binding::Map(v) =
                km.inner.get(key).ok_or(Error::KeySequenceNotFound(kseq.clone()))?
            {
                v
            } else {
                return Err(Error::KeySequenceNotFound(kseq.clone()));
            }
        }

        km.inner.get(last).ok_or(Error::KeySequenceNotFound(kseq.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut km = KeyMap::<i32>::new();

        {
            km.add("C-a b", 0).unwrap();
            assert!(matches!(km.lookup("C-a b").unwrap(), Binding::Value(0)));
            assert!(matches!(km.lookup("C-a").unwrap(), Binding::Map(_)));

            km.add("C-a b", 1).unwrap();
            assert!(matches!(km.lookup("C-a b").unwrap(), Binding::Value(1)));

            km.remove("C-a b").unwrap();
            assert!(matches!(km.lookup("C-a").unwrap(), Binding::Map(_)));
        }
    }
}
