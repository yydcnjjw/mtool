mod complete;

use std::fmt;

pub use complete::*;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        use crate::ui::wgui;
        use std::sync::Arc;
    }
}

pub enum Completion {
    #[cfg(not(target_family = "wasm"))]
    WGui(Arc<wgui::Completion>),
}

impl fmt::Debug for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Completion").finish()
    }
}

impl Completion {
    pub async fn complete_read<T>(
        &self,
        #[allow(unused)]
        args: CompletionArgs<T>,
    ) -> Result<Option<T>, anyhow::Error>
    where
        T: CompleteItem,
    {
        match self {
            #[cfg(not(target_family = "wasm"))]
            Completion::WGui(c) => c.complete_read(args).await,
            #[allow(unreachable_patterns)]
            _ => Ok(None),
        }
    }
}
