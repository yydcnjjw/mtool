use ::bevy::log::warn;
use dioxus_desktop::WindowAttributes;
use mapp::{
    anyhow, define_label,
    prelude::*,
    tokio::{self, task::JoinHandle},
};
use mtool_core::{AppStage, CmdlineStage, ConfigStore};
use mtool_storage::kv;

use crate::{
    bevy, builder::DioxusBuilder, custom_protocol::file_handler, main_view::main_view,
    router::Router,
};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("dioxus");

    group.add_module(Module);
    group.add_module(bevy::Module);

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
        ctx.schedule()
            .insert_stage_vec(
                CmdlineStage::AfterParse,
                vec![DioxusStage::Setup, DioxusStage::Launch],
            )
            .add_once_task(DioxusStage::Launch, launch)
            .add_once_task(AppStage::Run, wait_for_exit);

        ctx.injector().insert(Res::new(DioxusBuilder::new()));
        ctx.injector().insert(Res::new(Router::new()));
        Ok(())
    }
}

struct Worker(JoinHandle<()>);

async fn launch(
    injector: Injector,
    cs: Res<ConfigStore>,
    builder: Take<Res<DioxusBuilder>>,
    router: Res<Router>,
    kvstore: Res<kv::Store>,
) -> Result<(), anyhow::Error> {
    let event_loop_proxy = injector.construct_oneshot();

    let worker = {
        let builder = builder.take()?;

        let data_dir = cs.root_path().await;

        tokio::task::spawn_blocking(move || {
            builder
                .with_launch_builder(move |builder| {
                    builder
                        .with_context((*router).clone())
                        .with_context((*kvstore).clone())
                })
                .with_config_builder(move |cfg| {
                    use dioxus_desktop::winit::event_loop::EventLoop;

                    #[cfg(windows)]
                    use dioxus_desktop::winit::platform::windows::{
                        EventLoopBuilderExtWindows, WindowAttributesExtWindows,
                    };

                    #[cfg(unix)]
                    use dioxus_desktop::winit::platform::unix::EventLoopBuilderExtUnix;

                    let event_loop = EventLoop::with_user_event()
                        .with_any_thread(true)
                        .build()
                        .unwrap();

                    _ = event_loop_proxy.send(Res::new(event_loop.create_proxy()));

                    cfg.with_data_directory(data_dir)
                        .with_asynchronous_custom_protocol("mfile", file_handler)
                        .with_event_loop(event_loop)
                        .with_window(
                            WindowAttributes::default()
                                .with_decorations(false)
                                .with_transparent(true)
                                .with_skip_taskbar(true),
                        )
                })
                .launch(main_view)
        })
    };

    injector.insert(Res::new(Worker(worker)));

    Ok(())
}

async fn wait_for_exit(worker: Take<Res<Worker>>) -> Result<(), anyhow::Error> {
    Res::try_unwrap(worker.take()?)?.0.await?;
    Ok(())
}
