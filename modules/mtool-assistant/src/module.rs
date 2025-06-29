use ::bevy::{prelude::*, window::PresentMode};
use mapp::{anyhow, prelude::*, provider::Res as AppRes, tokio};
use mtool_cmdpal::{Command, CommandItem, CommandPalette};
use mtool_core::AppStage;
use mtool_dioxus::{
    bevy::{self, BevyAppBuilder},
    prelude::*,
};

use crate::{
    context::BevyState,
    emacs::{capture_inbox, capture_project},
    view,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-assistant");
    group.add_module(Module);
    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(AppRes::new(BevyState::new()));
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, setup_bevy)
            .add_once_task(AppStage::Init, init);
        Ok(())
    }
}

async fn open_window(ctx: AppRes<BevyState>) -> Result<(), anyhow::Error> {
    let (window, id) = bevy::create_window(Window {
        title: "My Assistant".to_owned(),
        present_mode: PresentMode::AutoVsync,
        visible: true,
        transparent: true,
        has_shadow: false,
        clip_children: false,
        ..default()
    })
    .await?;

    ctx.set_window_id(id);

    // attach_webview(window, view::assistant).await?;

    Ok(())
}

async fn init(ctx: AppRes<BevyState>, cmdpal: AppRes<CommandPalette>) -> Result<(), anyhow::Error> {
    // open_window(ctx).await?;

    cmdpal
        .add_top_level_command(CommandItem::from(
            Command::new("Capture project", capture_project).description("Capture project"),
        ))
        .add_top_level_command(CommandItem::from(
            Command::new("Capture inbox", capture_inbox).description("Capture inbox"),
        ));

    Ok(())
}

async fn setup_bevy(
    bevy_builder: AppRes<BevyAppBuilder>,
    builder: AppRes<DioxusBuilder>,
    ctx: AppRes<BevyState>,
) -> Result<(), anyhow::Error> {
    builder.add_global_hotkey("alt+c", || {
        tokio::spawn(async move {
            if let Err(e) = capture_inbox().await {
                warn!("{:?}", e);
            }
        });
        Ok(())
    });

    bevy_builder.on_setup(move |app| crate::bevy::assistant(app, ctx));
    Ok(())
}
