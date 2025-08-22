use dioxus::prelude::*;
use mapp::{anyhow, futures::TryFutureExt, prelude::*, tokio, tracing::warn};
use mtool_cmdpal::{Command, CommandItem, CommandPalette};
use mtool_dioxus::{
    desktop::{winit::window::WindowLevel, WindowAttributes},
    prelude::*,
};

use crate::{
    emacs::{capture_inbox, capture_project},
    media, notify, view,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-assistant");
    group
        .add_module(Module)
        .add_module(media::Module)
        .add_module(notify::Module);

    #[cfg(feature = "bevy")]
    group.add_module(crate::bevy::Module);

    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(DioxusStage::Setup, setup);
        #[cfg(feature = "desktop")]
        {
            use mtool_core::AppStage;
            ctx.schedule().add_once_task(AppStage::Init, init);
        }

        Ok(())
    }
}

async fn setup(
    #[cfg(target_os = "android")] router: Res<Router>,
    builder: Res<DioxusBuilder>,
) -> Result<(), anyhow::Error> {
    builder.add_global_hotkey("alt+c", || {
        tokio::spawn(async move {
            if let Err(e) = capture_inbox().await {
                warn!("{:?}", e);
            }
        });
        Ok(())
    });

    _ = tokio::spawn(create_window().inspect_err(|e| warn!("{e:?}")));

    #[cfg(target_os = "android")]
    {
        mtool_dioxus::add_route!(router, "assistant", view::MainView);
        router.route("assistant");
    }

    Ok(())
}

async fn create_window() -> Result<(), anyhow::Error> {
    let window_attrs = WindowAttributes::default()
        .with_title("Mtool assistant")
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_visible(false);

    #[cfg(target_os = "windows")]
    let window_attrs = {
        use mtool_dioxus::desktop::winit::platform::windows::{
            CornerPreference, WindowAttributesExtWindows,
        };
        window_attrs
            .with_skip_taskbar(false)
            .with_undecorated_shadow(true)
            .with_corner_preference(CornerPreference::Round)
    };

    spawn_window(WebviewWindowConfig::new(view::MainView).with_window_attributes(window_attrs))
        .await
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
