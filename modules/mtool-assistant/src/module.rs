#[allow(unused)]
use dioxus::prelude::*;
use mapp::{anyhow, futures::TryFutureExt, prelude::*, tokio, tracing::warn};
#[cfg(feature = "desktop")]
use mtool_cmdpal::{Command, CommandItem, CommandPalette};
use mtool_core::ConfigStore;
use mtool_dioxus::{
    desktop::{winit::window::WindowLevel, WindowAttributes},
    prelude::*,
};

use crate::{emacs, media, notify, view};

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
    cs: Res<ConfigStore>,
) -> Result<(), anyhow::Error> {
    builder.add_global_hotkey("alt+c", || {
        tokio::spawn(async move {
            if let Err(e) = emacs::capture_inbox().await {
                warn!("{:?}", e);
            }
        });
        Ok(())
    });

    _ = tokio::spawn(create_window(cs).inspect_err(|e| warn!("{e:?}")));

    #[cfg(target_os = "android")]
    {
        mtool_dioxus::add_route!(router, "assistant", view::MainView);
        router.route("assistant");
    }

    Ok(())
}

async fn create_window(cs: Res<ConfigStore>) -> Result<(), anyhow::Error> {
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
            .with_skip_taskbar(true)
            .with_corner_preference(CornerPreference::Round)
    };

    spawn_window(
        WebviewWindowConfig::new(view::MainView)
            .with_data_directory(cs.root_path())
            // .with_background_color((0, 0, 0, 0))
            .with_window_attributes(window_attrs),
    )
    .await
}

#[cfg(feature = "desktop")]
async fn init(cmdpal: Res<CommandPalette>) -> Result<(), anyhow::Error> {
    cmdpal
        .add_top_level_command(CommandItem::from(
            Command::new("Capture project", emacs::capture_project).description("Capture project"),
        ))
        .add_top_level_command(CommandItem::from(
            Command::new("Capture inbox", emacs::capture_inbox).description("Capture inbox"),
        ));

    Ok(())
}
