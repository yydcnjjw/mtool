use bevy::{prelude::*, window::PresentMode};
use dioxus::prelude::*;
use mapp::{
    anyhow::{self},
    prelude::*,
    provider::Res as AppRes,
};
use mtool_core::AppStage;
use mtool_dioxus::{
    bevy::{create_window, BevyAppBuilder},
    prelude::*,
};

use super::{state::BevyState, system::assistant};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(AppRes::new(BevyState::new()));
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, setup)
            .add_once_task(AppStage::Init, init);
        Ok(())
    }
}

async fn open_window(ctx: AppRes<BevyState>) -> Result<(), anyhow::Error> {
    let (window, id) = create_window(Window {
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

async fn init(ctx: AppRes<BevyState>) -> Result<(), anyhow::Error> {
    // open_window(ctx).await?;
    Ok(())
}

async fn setup(
    bevy_builder: AppRes<BevyAppBuilder>,
    builder: AppRes<DioxusBuilder>,
    ctx: AppRes<BevyState>,
) -> Result<(), anyhow::Error> {
    bevy_builder.on_setup(move |app| assistant(app, ctx));
    Ok(())
}
