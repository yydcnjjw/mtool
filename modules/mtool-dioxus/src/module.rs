use std::sync::Arc;

use dioxus_desktop::{wry, WindowAttributes};
use mapp::{
    anyhow::{self, anyhow},
    define_label,
    prelude::*,
    tokio::{self, sync::oneshot, task::JoinHandle},
    tracing::{debug, info, warn},
};
use mtool_core::{AppStage, CmdlineStage, ConfigStore};
use mtool_storage::kv;

use crate::{
    // bevy,
    builder::DioxusBuilder,
    custom_protocol::file_handler,
    main_view::main_view,
    router::Router,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("dioxus");

    group.add_module(Module);
    // group.add_module(bevy::Module);

    group
}

struct Module;

define_label!(
    pub enum DioxusStage {
        Setup,
        Launch,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        ctx.injector().insert(Arc::new(tx));

        ctx.schedule()
            .insert_stage_vec(
                CmdlineStage::AfterParse,
                vec![DioxusStage::Setup, DioxusStage::Launch],
            )
            .add_once_task(DioxusStage::Launch, launch)
            .setup_main_thread_loop(move || main_loop(rx));

        ctx.injector().insert(Res::new(DioxusBuilder::new()));
        ctx.injector().insert(Res::new(Router::new()));
        Ok(())
    }
}

type MainLoopRunner = Box<dyn FnOnce() -> Result<(), anyhow::Error> + Send>;

#[cfg(target_os = "android")]
use dioxus_desktop::winit::platform::android::activity::AndroidApp;

async fn launch(
    injector: Injector,
    cs: Res<ConfigStore>,
    builder: Take<Res<DioxusBuilder>>,
    router: Res<Router>,
    kvstore: Res<kv::Store>,
    #[cfg(target_os = "android")] android_app: Res<AndroidApp>,
    runner: Take<Arc<oneshot::Sender<MainLoopRunner>>>,
) -> Result<(), anyhow::Error> {
    let event_loop_proxy = injector.construct_oneshot();

    let builder = builder.take()?;

    let data_dir = cs.root_path().await;

    if let Err(_) = Arc::try_unwrap(runner.take()?)
        .map_err(|e| anyhow!("Arc::try_unwrap MainLooperRunner"))?
        .send(Box::new(move || {
            info!("main thread loop is running");

            builder
                .with_launch_builder(move |builder| {
                    builder
                        .with_context((*router).clone())
                        .with_context((*kvstore).clone())
                })
                .with_config_builder(move |cfg| {
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
                    let event_loop = {
                        use dioxus_desktop::winit::platform::android::EventLoopBuilderExtAndroid;
                        use std::ops::Deref;

                        android_fn![org_yydcnjjw_mtool, wry, WryActivity, onCreate, [JObject]];
                        wry::android_binding!(org_yydcnjjw_mtool, wry, wry);

                        let android_app = android_app.deref().clone();

                        EventLoop::with_user_event()
                            .with_android_app(android_app)
                            .build()
                            .unwrap()
                    };

                    _ = event_loop_proxy.send(Res::new(event_loop.create_proxy()));

                    let mut window_attrs = WindowAttributes::default()
                        .with_decorations(false)
                        .with_transparent(true);

                    #[cfg(target_os = "windows")]
                    {
                        window_attrs = window_attrs.with_skip_taskbar(true);
                    }

                    cfg.with_data_directory(data_dir)
                        .with_asynchronous_custom_protocol("mfile", file_handler)
                        .with_event_loop(event_loop)
                        .with_window(window_attrs)
                })
                .launch(main_view);
            Ok(())
        }))
    {
        warn!("failed to send runner");
    }

    Ok(())
}

fn main_loop(runner: oneshot::Receiver<MainLoopRunner>) -> Result<(), anyhow::Error> {
    debug!("main_loop is startup");

    runner.blocking_recv()?()
}

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub unsafe fn onCreate(jenv: JNIEnv, _: JClass, activity: JObject) {
    let activity = jenv.new_global_ref(activity).unwrap();

    dioxus_desktop::android_setup(
        "org/yydcnjjw/mtool/wry",
        jenv,
        &ndk::looper::ThreadLooper::for_thread().unwrap(),
        activity,
    );
}
