use ::bevy::{
    prelude::*,
    window::{CompositeAlphaMode, PresentMode},
};
use mapp::{anyhow, prelude::*, provider::Res as AppRes};
use mtool_cmdpal::{CommandPalette, CommandResult};
use mtool_core::AppStage;
use mtool_dioxus::{bevy, prelude::*};

use crate::view;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-pdf");
    group.add_module(Module);
    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(AppStage::Init, init);
        Ok(())
    }
}

async fn open_pdf_viewer() -> Result<CommandResult, anyhow::Error> {
    let window = bevy::create_window(Window {
        title: "pdf viewer".to_owned(),
        visible: true,
        transparent: true,
        has_shadow: false,
        clip_children: false,
        ..default()
    })
    .await?;

    attach_webview(window, view::pdf_viewer).await?;

    Ok(CommandResult::Dismiss)
}

async fn init(cmdpal: AppRes<CommandPalette>) -> Result<(), anyhow::Error> {
    open_pdf_viewer().await?;
    // cmdpal.add_top_level_command(CommandItem::from(Arc::new(
    //     Command::new("open pdf viewer", move || {
    //         open_pdf_viewer(event_loop.clone())
    //     })
    //     .description("open pdf viewer"),
    // )));
    Ok(())
}
