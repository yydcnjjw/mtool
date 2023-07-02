cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod dict;
        mod entities;
        pub use dict::*;
    }
}

mod view;
pub use view::*;

use async_trait::async_trait;
use mapp::prelude::*;

pub struct Module;

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(Dict::construct);
        Ok(())
    }
}


#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        use mapp::provider::Res;
        use mtool_wgui::{WebStage, Templator};
        ctx.schedule()
            .add_once_task(WebStage::Init, |templator: Res<Templator>| async move {
                templator.add_template::<DictView>();
                Ok::<(), anyhow::Error>(())
            });

        Ok(())
    }
}
