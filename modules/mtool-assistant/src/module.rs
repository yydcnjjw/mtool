use std::sync::{Arc, Once};

use dioxus::prelude::*;
use mapp::{
    anyhow,
    prelude::*,
    tokio,
    tracing::{error, warn},
};
use mtool_cmdpal::{Command, CommandItem, CommandPalette};
use mtool_core::AppStage;
use mtool_dioxus::{
    add_route,
    desktop::{
        winit::{
            event::{Event, StartCause},
            event_loop::ActiveEventLoop,
        },
        WindowAttributes,
    },
    prelude::*,
};

use crate::{
    components,
    context::{self, AssistantContext},
    emacs::{capture_inbox, capture_project},
    notify,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-assistant");
    group.add_module(Module).add_module(notify::Module);

    #[cfg(feature = "bevy")]
    group.add_module(crate::bevy::Module);

    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(AssistantContext::construct);
        ctx.schedule().add_once_task(DioxusStage::Setup, setup);
        #[cfg(feature = "desktop")]
        {
            ctx.schedule()
                .add_once_task(AppStage::Init, init)
                .add_once_task(AppStage::Init, context::register_commands);
        }

        Ok(())
    }
}

async fn setup(
    #[cfg(target_os = "android")] router: Res<Router>,
    builder: Res<DioxusBuilder>,
    context: Res<AssistantContext>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    builder.add_global_hotkey("alt+c", || {
        tokio::spawn(async move {
            if let Err(e) = capture_inbox().await {
                warn!("{:?}", e);
            }
        });
        Ok(())
    });

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        to_owned![context];
        builder.with_config_builder(move |cfg| {
            cfg.with_custom_event_handler(move |ev, event_loop| match ev {
                Event::NewEvents(StartCause::Init) => {
                    static INIT: Once = Once::new();
                    to_owned![context, injector];
                    INIT.call_once(move || {
                        if let Err(e) = create_window(event_loop, context, injector) {
                            error!("{:?}", e);
                            event_loop.exit();
                        }
                    });
                }
                _ => {}
            })
        });
    }

    #[cfg(target_os = "android")]
    {
        builder.with_launch_builder(|builder| builder.with_context(context));

        add_route!(router, "assistant", components::MainView);
        router.route("assistant");
    }

    Ok(())
}

fn create_window(
    event_loop: &ActiveEventLoop,
    context: Res<AssistantContext>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    let window = event_loop.create_window(
        WindowAttributes::default()
            .with_title("Mtool assistant")
            .with_visible(false),
    )?;

    tokio::spawn(async move {
        if let Err(e) = WinitWebviewBuilder::new(Arc::new(window), components::MainView)
            .with_context(context)
            .with_context(injector.get::<Res<DioxusContext>>().await.unwrap())
            .build()
            .await
        {
            warn!("{:?}", e);
        }
    });
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
