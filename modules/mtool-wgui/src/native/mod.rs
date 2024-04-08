mod builder;
mod global_hotkey;
mod window;
mod window_data_bind;

pub use builder::*;
pub use window::{MtoolWindow, WGuiWindow};
pub use window_data_bind::WindowDataBind;

use async_trait::async_trait;
use mapp::{define_label, prelude::*};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage, CmdlineStage,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
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

pub struct Module<R: tauri::Runtime> {
    tauri_context: Mutex<Option<tauri::Context<R>>>,
}

impl<R: tauri::Runtime> Module<R> {
    pub fn new(tauri_context: tauri::Context<R>) -> Self {
        Self {
            tauri_context: Mutex::new(Some(tauri_context)),
        }
    }
}

#[async_trait]
impl<R> AppModule for Module<R>
where
    R: tauri::Runtime,
{
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Builder::<R>::new);
        app.injector()
            .insert(Take::new(self.tauri_context.lock().await.take().unwrap()));

        app.schedule()
            .insert_stage_vec_with_cond(
                CmdlineStage::AfterInit,
                vec![WGuiStage::Setup, WGuiStage::Init, WGuiStage::AfterInit],
                is_startup_mode(StartupMode::WGui),
            )
            .add_once_task(WGuiStage::Setup, setup::<R>)
            .add_once_task(WGuiStage::Init, init::<R>)
            .add_once_task(AppStage::Run, wait_for_exit);

        Ok(())
    }
}

pub fn module<R>(tauri_context: tauri::Context<R>) -> ModuleGroup
where
    R: tauri::Runtime,
{
    let mut group = ModuleGroup::new("mtool-wgui-native");
    group.add_module(Module::new(tauri_context));

    #[cfg(windows)]
    group.add_module(global_hotkey::Module);
    group
}

async fn setup<R>(builder: Res<Builder<R>>, injector: Injector) -> Result<(), anyhow::Error>
where
    R: tauri::Runtime,
{
    let app_tx = injector.construct_oneshot();
    let mtool_win_tx: oneshot::Sender<Res<MtoolWindow<R>>> = injector.construct_oneshot();

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

            app_tx.send(Res::new(app.clone())).unwrap();
            Ok(())
        })
        .setup(move |builder| Ok(builder.plugin(window::init::<R>(mtool_win_tx))))?;

    Ok(())
}

struct TauriWorker(tokio::task::JoinHandle<()>);

async fn init<R: tauri::Runtime>(
    builder: Res<Builder<R>>,
    injector: Injector,
    tauri_context: Take<tauri::Context<R>>,
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
