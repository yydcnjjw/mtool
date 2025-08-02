use std::sync::Arc;

use mapp::{anyhow, define_label, prelude::*, tokio::sync::oneshot};
use mtool_core::CmdlineStage;

use crate::{
    builder::DioxusBuilder,
    launch::{launch, main_loop},
    router::Router,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("dioxus");

    group.add_module(Module);
    #[cfg(feature = "bevy")]
    group.add_module(crate::bevy::Module);

    group
}

struct Module;

define_label!(
    pub enum DioxusStage {
        Setup,
        Launch,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        ctx.injector().insert(Arc::new(tx));

        ctx.schedule()
            .insert_stage_vec(
                CmdlineStage::AfterParse,
                vec![DioxusStage::Setup, DioxusStage::Launch],
            )
            .add_once_task(DioxusStage::Launch, launch)
            .setup_main_thread_loop(move || main_loop(rx));

        ctx.injector().insert(Res::new(DioxusBuilder::new()));
        ctx.injector().insert(Res::new(Router::new()));
        Ok(())
    }
}
