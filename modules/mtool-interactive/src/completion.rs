use std::{fmt, sync::Arc};

use crate::{wgui, CompleteItem, CompleteRead, CompletionArgs};

pub enum Completion {
    WGui(Arc<wgui::Completion>),
}

impl fmt::Debug for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Completion").finish()
    }
}

impl Completion {
    pub async fn complete_read<T>(&self, args: CompletionArgs<T>) -> Result<T, anyhow::Error>
    where
        T: CompleteItem,
    {
        match self {
            Completion::WGui(c) => c.complete_read(args).await,
        }
    }
}
