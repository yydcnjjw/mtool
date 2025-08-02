use dioxus::prelude::*;
use mapp::{
    anyhow::{self},
    prelude::*,
    tokio,
    tracing::warn,
};
use mtool_cmdpal::{Command, CommandItem, CommandPalette};
use mtool_core::AppStage;
use mtool_dioxus::{add_route, prelude::*};

use crate::{
    components,
    emacs::{capture_inbox, capture_project},
    notify::{self, NotifyContext},
    rpc,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-assistant");
    group
        .add_module(Module)
        .add_module(rpc::Module)
        .add_module(notify::Module);

    #[cfg(feature = "bevy")]
    group.add_module(crate::bevy::Module);

    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, setup)
            .add_once_task(AppStage::Init, init);
        Ok(())
    }
}

async fn setup(
    router: Res<Router>,
    builder: Res<DioxusBuilder>,
    notify_context: Res<NotifyContext>,
) -> Result<(), anyhow::Error> {
    builder.add_global_hotkey("alt+c", || {
        tokio::spawn(async move {
            if let Err(e) = capture_inbox().await {
                warn!("{:?}", e);
            }
        });
        Ok(())
    });

    builder.with_launch_builder(move |builder| builder.with_context(notify_context));

    add_route!(router, "assistant", components::MainView);

    #[cfg(any(target_os = "android"))]
    router.route("assistant");

    Ok(())
}

async fn init(cmdpal: Res<CommandPalette>) -> Result<(), anyhow::Error> {
    cmdpal
        .add_top_level_command(CommandItem::from(
            Command::new("Capture project", capture_project).description("Capture project"),
        ))
        .add_top_level_command(CommandItem::from(
            Command::new("Capture inbox", capture_inbox).description("Capture inbox"),
        ));

    Ok(())
}
