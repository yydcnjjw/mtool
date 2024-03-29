mod builder;
mod global_hotkey;
mod window;
mod window_data_bind;

pub use builder::*;
pub use window::{MtoolWindow, WGuiWindow};
pub use window_data_bind::WindowDataBind;

use async_trait::async_trait;
use mapp::{define_label, prelude::*, CreateOnceTaskDescriptor};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage, CmdlineStage,
};
use mtool_system::keybinding::Keybinding;
use tauri::{
    menu::{Menu, MenuId, MenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent,
};
use tokio::sync::{oneshot, Mutex};
use tracing::{debug, info, warn};

define_label! {
    pub enum WGuiStage {
        Setup,
        Init,
        AfterInit,
    }
}

pub struct Module<A: tauri::Assets> {
    tauri_context: Mutex<Option<tauri::Context<A>>>,
}

impl<A: tauri::Assets> Module<A> {
    pub fn new(tauri_context: tauri::Context<A>) -> Self {
        Self {
            tauri_context: Mutex::new(Some(tauri_context)),
        }
    }
}

#[async_trait]
impl<A> AppModule for Module<A>
where
    A: tauri::Assets,
{
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Builder::new);
        app.injector()
            .insert(Take::new(self.tauri_context.lock().await.take().unwrap()));

        app.schedule()
            .insert_stage_vec_with_cond(
                CmdlineStage::AfterInit,
                vec![WGuiStage::Setup, WGuiStage::Init, WGuiStage::AfterInit],
                is_startup_mode(StartupMode::WGui),
            )
            .add_once_task(WGuiStage::Setup, setup)
            .add_once_task(WGuiStage::Init, init::<A>)
            .add_once_task(
                AppStage::Init,
                register_keybinding.cond(is_startup_mode(StartupMode::WGui)),
            )
            .add_once_task(AppStage::Run, wait_for_exit);

        Ok(())
    }
}

pub fn module<A>(tauri_context: tauri::Context<A>) -> ModuleGroup
where
    A: tauri::Assets,
{
    let mut group = ModuleGroup::new("mtool-wgui-native");
    group.add_module(Module::new(tauri_context));

    #[cfg(windows)]
    group.add_module(global_hotkey::Module);
    group
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    let (tx, rx) = oneshot::channel();

    injector.construct_once(|| async move { Ok(rx.await?) });

    builder
        .setup_with_app(move |app| {
            let app = app.handle();
            {
                let menu = Menu::with_items(
                    app,
                    &[&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?],
                )?;
                let builder = TrayIconBuilder::with_id("mtool")
                    .tooltip("MTool")
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu);

                // HACK: for keepalive
                app.manage(menu);
                builder
                    .menu_on_left_click(false)
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        _ => (),
                    })
                    .build(app)?;
            }

            tx.send(Res::new(app.clone())).unwrap();
            Ok(())
        })
        .setup(move |builder| Ok(builder.plugin(window::init(injector))))?;

    Ok(())
}

struct TauriWorker(tokio::task::JoinHandle<()>);

async fn init<A: tauri::Assets>(
    builder: Res<Builder>,
    injector: Injector,
    tauri_context: Take<tauri::Context<A>>,
) -> Result<(), anyhow::Error> {
    let builder = builder.take();

    let worker = tokio::task::spawn_blocking(move || {
        debug!("tauri run at {:?}", std::thread::current().name());

        match builder.any_thread().build(tauri_context.take().unwrap()) {
            Ok(v) => v,
            Err(e) => {
                warn!("tauri run loop is exited: {:?}", e);
                return;
            }
        }
        .run(move |_, ev| match ev {
            _ => {}
        });
        info!("tauri run loop is exited");
    });

    injector.insert(Take::new(TauriWorker(worker)));

    Ok(())
}

async fn wait_for_exit(worker: TakeOpt<TauriWorker>) -> Result<(), anyhow::Error> {
    if let Some(worker) = worker.unwrap() {
        worker.take()?.0.await?;
    }

    Ok(())
}

async fn register_keybinding(keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
    keybinding
        .define_global("M-A-o", window::show_window)
        .await?;
    keybinding
        .define_global("M-A-S-o", window::hide_window)
        .await?;
    Ok(())
}
