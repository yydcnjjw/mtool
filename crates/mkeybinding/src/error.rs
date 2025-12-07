use mapp::anyhow;
use thiserror::Error;

use crate::KeySequence;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Key sequence not found {0}")]
    KeySequenceNotFound(KeySequence),
    #[error("Key sequence {kseq} starts with non-prefix key {prefix}")]
    KeySequenceExisted {
        kseq: KeySequence,
        prefix: KeySequence,
    },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
