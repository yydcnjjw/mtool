cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod dict;
        pub use dict::*;
    }
}

mod view;
pub use view::*;

use async_trait::async_trait;
use mapp::{AppContext, AppModule};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        #[cfg(not(target_family = "wasm"))]
        {
            ctx.injector().construct_once(Dict::construct);
        }

        #[cfg(target_family = "wasm")]
        {
            use mapp::provider::Res;
            use mtool_wgui::{AppStage, Templator};
            ctx.schedule()
                .add_once_task(AppStage::Init, |templator: Res<Templator>| async move {
                    templator.add_template::<DictView>();
                    Ok::<(), anyhow::Error>(())
                });
        }

        Ok(())
    }
}
