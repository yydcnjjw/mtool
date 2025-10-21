use std::{any::type_name_of_val, sync::Arc};

use dioxus_desktop::{UserWindowEvent, WindowAttributes};
use mapp::{
    anyhow,
    prelude::*,
    tokio::sync::oneshot,
    tracing::{debug, info, warn},
};
use mtool_core::ConfigStore;

use crate::{
    builder::DioxusBuilder, context::DioxusContext, custom_protocol::file_handler,
    main_view::main_view,
};

type MainLoopRunner = Box<dyn FnOnce() -> Result<(), anyhow::Error> + Send>;


#[cfg(target_os = "android")]
use crate::winit::{event::Event as WinitEvent, platform::android::activity::AndroidApp};

pub(crate) async fn launch(
    injector: Injector,
    cs: Res<ConfigStore>,
    builder: Take<Res<DioxusBuilder>>,
    exit_signal: Res<ExitSignal>,
    #[cfg(target_os = "android")] android_app: Res<AndroidApp>,
    runner: Take<Arc<oneshot::Sender<MainLoopRunner>>>,
) -> Result<(), anyhow::Error> {
    let builder = builder.take()?;

    let data_dir = cs.root_path();

    let context_tx = injector.construct_oneshot();

    if let Err(_) = runner.take()?.send(Box::new(move || {
        info!("main thread loop is running");

        builder
            .with_dioxus(move |cfg, builder| {
                use dioxus_desktop::winit::event_loop::EventLoop;

                #[cfg(target_os = "windows")]
                use dioxus_desktop::winit::platform::windows::{
                    EventLoopBuilderExtWindows, WindowAttributesExtWindows,
                };

                #[cfg(target_os = "linux")]
                use dioxus_desktop::winit::platform::unix::EventLoopBuilderExtUnix;

                #[cfg(any(target_os = "linux", target_os = "windows"))]
                let event_loop = EventLoop::with_user_event()
                    .with_any_thread(true)
                    .build()
                    .unwrap();

                #[cfg(target_os = "android")]
                let (event_loop, android_app) = {
                    use dioxus_desktop::{
                        winit::platform::android::EventLoopBuilderExtAndroid,
                        wry::{self, prelude::*},
                    };
                    use std::ops::Deref;

                    wry::android_binding!(org_yydcnjjw_mtool_dioxus, wry, wry);

                    let android_app = android_app.deref().clone();

                    (
                        EventLoop::with_user_event()
                            .with_android_app(android_app.clone())
                            .build()
                            .unwrap(),
                        android_app,
                    )
                };

                let (context, event_loop_context) = DioxusContext::new(
                    event_loop.create_proxy(),
                    #[cfg(target_os = "android")]
                    android_app,
                );

                if let Err(e) = context_tx.send(Res::new(context.clone())) {
                    warn!("Failed to send {}", type_name_of_val(&e));
                }

                {
                    let event_loop = event_loop.create_proxy();
                    exit_signal.add_handler(move || {
                        if let Err(e) = event_loop.send_event(UserWindowEvent::Shutdown) {
                            warn!("{:?}", e);
                        }
                    });
                }

                #[allow(unused_mut)]
                let mut window_attrs = WindowAttributes::default()
                    .with_decorations(false)
                    .with_transparent(true);

                #[cfg(target_os = "windows")]
                {
                    use dioxus_desktop::winit::platform::windows::CornerPreference;
                    window_attrs = window_attrs
                        .with_skip_taskbar(true)
                        .with_corner_preference(CornerPreference::Round);
                }

                (
                    cfg.with_data_directory(data_dir)
                        .with_asynchronous_custom_protocol("mfile", file_handler)
                        .with_event_loop(event_loop)
                        .with_window(window_attrs)
                        .with_custom_event_handler(move |event, event_loop| match event {
                            WinitEvent::UserEvent(UserWindowEvent::WakeUp) => {
                                event_loop_context.pool_events(event_loop);
                            }
                            _ => {}
                        }),
                    builder.with_context(context).with_context(injector),
                )
            })
            .launch(main_view);
        Ok(())
    })) {
        warn!("failed to send runner");
    }

    Ok(())
}

pub(crate) fn main_loop(runner: oneshot::Receiver<MainLoopRunner>) -> Result<(), anyhow::Error> {
    debug!("main_loop is startup");

    runner.blocking_recv()?()
}

