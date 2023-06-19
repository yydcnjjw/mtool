mod complete;

use std::fmt;

use async_trait::async_trait;
pub use complete::*;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_wgui::Templator;

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

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        use mtool_wgui::AppStage;
        ctx.schedule()
            .add_once_task(AppStage::Init, |templator: Res<Templator>| async move {
                templator.add_template::<TextCompleteItemView>();
                Ok::<(), anyhow::Error>(())
            });
        Ok(())
    }
}
