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
    emacs::{capture_inbox, capture_project},
    music,
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
        ctx.schedule().add_once_task(DioxusStage::Setup, setup);
        #[cfg(feature = "desktop")]
        ctx.schedule().add_once_task(AppStage::Init, init);
        Ok(())
    }
}

async fn setup(
    router: Res<Router>,
    builder: Res<DioxusBuilder>,
    notify_context: Res<NotifyContext>,
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
        to_owned![notify_context];
        builder.with_config_builder(move |cfg| {
            cfg.with_custom_event_handler(move |ev, event_loop| match ev {
                Event::NewEvents(StartCause::Init) => {
                    static INIT: Once = Once::new();
                    to_owned![notify_context, injector];
                    INIT.call_once(move || {
                        if let Err(e) = create_window(event_loop, notify_context, injector) {
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
        builder.with_launch_builder(|builder| builder.with_context(notify_context));

        add_route!(router, "assistant", components::MainView);
        router.route("assistant");
    }

    Ok(())
}

fn create_window(
    event_loop: &ActiveEventLoop,
    notify_context: Res<NotifyContext>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    let window = event_loop.create_window(
        WindowAttributes::default()
            .with_title("Mtool assistant")
            .with_visible(false),
    )?;

    tokio::spawn(async move {
        if let Err(e) = WinitWebviewBuilder::new(Arc::new(window), components::MainView)
            .with_context(notify_context)
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
