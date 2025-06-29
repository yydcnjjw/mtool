use bevy::{
    app::PluginsState,
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{PowerPreference, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::ExitCondition,
    winit::{DisplayHandleWrapper, EventLoopProxyWrapper},
};
use dioxus_desktop::winit::{application::ApplicationHandler, event::Event as WInitEvent};
use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    prelude::{Res as AppRes, *},
    tokio::sync::mpsc,
};

use crate::{builder::DioxusBuilder, module::DioxusStage};

use super::{
    event::BevyUserEvent, receive_create_window_event, set_window_sender, try_send_window,
    BevyAppBuilder, WindowFactory,
};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(
            BEVY_BUILDER
                .get_or_init(|| AppRes::new(BevyAppBuilder::new()))
                .to_owned(),
        );
        ctx.schedule().add_once_task(DioxusStage::Setup, bevy_setup);
        Ok(())
    }
}

static BEVY_BUILDER: OnceCell<AppRes<BevyAppBuilder>> = OnceCell::new();

async fn bevy_setup(builder: AppRes<DioxusBuilder>) -> Result<(), anyhow::Error> {
    builder.with_launcher(|root, contexts, platform_config| {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..Default::default()
                })
                .set(AssetPlugin {
                    file_path: "/home/yydcnjjw/workspace/project/mtool/assets".to_owned(),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        // instance_flags: InstanceFlags::empty(),
                        // power_preference: PowerPreference::HighPerformance,
                        power_preference: PowerPreference::LowPower,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        // .insert_resource(ClearColor(Color::NONE))
            ;

        app.add_systems(Update, (receive_create_window_event, try_send_window));

        if let Some(builder) = BEVY_BUILDER.get() {
            if let Err(e) = builder.run_setup_hooks(&mut app) {
                panic!("{:?}", e);
            }
        }

        app.set_runner(move |mut app| {
            if app.plugins_state() == PluginsState::Ready {
                app.finish();
                app.cleanup();
            }

            let mut platform_config = *platform_config
                .into_iter()
                .find_map(|cfg| cfg.downcast::<dioxus_desktop::Config>().ok())
                .unwrap_or_default();

            if let Some(ref event_loop) = platform_config.event_loop {
                app.insert_resource(DisplayHandleWrapper(event_loop.owned_display_handle()));
                app.world_mut()
                    .insert_resource(EventLoopProxyWrapper(event_loop.create_proxy()));
            }

            {
                let (tx, rx) = mpsc::unbounded_channel();

                app.world_mut().insert_resource(WindowFactory::new(rx));

                set_window_sender(tx);
            };

            let mut runner_state =
                bevy::winit::state::WinitAppRunnerState::<BevyUserEvent>::new(app);

            platform_config =
                platform_config.with_custom_event_handler(move |event, event_loop| {
                    let app = &mut runner_state;
                    match event.clone() {
                        WInitEvent::NewEvents(cause) => app.new_events(event_loop, cause),
                        WInitEvent::WindowEvent { window_id, event } => {
                            app.window_event(event_loop, window_id, event)
                        }
                        WInitEvent::DeviceEvent { device_id, event } => {
                            app.device_event(event_loop, device_id, event)
                        }
                        WInitEvent::UserEvent(_) => {}
                        WInitEvent::Suspended => app.suspended(event_loop),
                        WInitEvent::Resumed => app.resumed(event_loop),
                        WInitEvent::AboutToWait => app.about_to_wait(event_loop),
                        WInitEvent::LoopExiting => app.exiting(event_loop),
                        WInitEvent::MemoryWarning => app.memory_warning(event_loop),
                    }
                });

            dioxus_desktop::launch::launch(root, contexts, vec![Box::new(platform_config)]);

            AppExit::Success
        });

        app.run();
    });
    Ok(())
}
