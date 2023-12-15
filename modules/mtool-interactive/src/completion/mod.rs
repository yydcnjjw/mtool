mod complete;
mod complete_item;

pub use complete::*;
pub use complete_item::*;

use async_trait::async_trait;
use mapp::prelude::*;
use mtool_wgui::Templator;
use std::fmt;

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
        #[allow(unused)] args: CompletionArgs<T>,
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

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        use mtool_wgui::WebStage;
        ctx.schedule()
            .add_once_task(WebStage::Init, |templator: Res<Templator>| async move {
                templator.add_template::<TextCompleteItemView>();
                templator.add_template::<PathBufCompleteItemView>();
                Ok::<(), anyhow::Error>(())
            });
        Ok(())
    }
}
