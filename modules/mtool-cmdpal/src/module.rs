use crate::{components, CommandPalette};
use dioxus::prelude::*;
use mapp::{anyhow, prelude::*};
use mtool_dioxus::{add_route, prelude::*};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-cmdpal");
    group.add_module(Module);
    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(CommandPalette::new()));
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, dioxus_setup);
        Ok(())
    }
}

async fn dioxus_setup(
    builder: Res<DioxusBuilder>,
    router: Res<Router>,
    cmd_mgr: Res<CommandPalette>,
) -> Result<(), anyhow::Error> {
    builder.with_launch_builder(move |builder| builder.with_context(cmd_mgr));

    add_route!(router, "cmdpal", components::CommandPaletteView);

    router.route("cmdpal");
    Ok(())
}
