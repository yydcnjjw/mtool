mod backend;
pub mod ecdict;
pub mod mdx;

pub use backend::*;

use mapp::prelude::*;

pub struct Module;

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(ecdict::Dict::construct);
        ctx.injector().construct_once(mdx::Dict::construct);
        Ok(())
    }
}

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        use mapp::provider::Res;
        use mtool_wgui::{Templator, WebStage};

        async fn setup_template(templator: Res<Templator>) -> Result<(), anyhow::Error> {
            templator.add_template::<ecdict::DictView>();
            templator.add_template::<mdx::DictView>();
            Ok(())
        }

        ctx.schedule().add_once_task(WebStage::Init, setup_template);

        Ok(())
    }
}
