use std::{fmt, ops::Deref, sync::Arc};

use crate::complete::CompleteRead;

pub type SharedCompletion = Arc<dyn CompleteRead + Send + Sync>;

pub struct Completion(pub SharedCompletion);

impl fmt::Debug for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Completion").finish()
    }
}

impl Completion {}

impl Deref for Completion {
    type Target = SharedCompletion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
