pub mod completion;
mod interactive_window;
mod output;

pub use completion::Completion;
pub use interactive_window::*;
use mtool_gui::{Builder, GuiStage};
pub use output::OutputDevice;

use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(GuiStage::Setup, setup)
            .add_once_task(GuiStage::AfterInit, register_keybinding);

        Ok(())
    }
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    builder.setup(|builder| {
        Ok(builder
            .plugin(interactive_window::init(injector.clone()))
            .plugin(completion::init())
            .plugin(output::init()))
    })?;

    injector
        .construct_once(InteractiveWindow::new)
        .construct_once(Completion::new)
        .construct_once(OutputDevice::new);
    Ok(())
}

#[cfg(not(windows))]
use mtool_system::keybinding::Keybinging;

#[cfg(not(windows))]
async fn register_keybinding(keybinding: Res<Keybinging>) -> Result<(), anyhow::Error> {
    keybinding.define_global("M-A-q", interactive_window::hide_window)?;
    Ok(())
}

#[cfg(windows)]
use tauri::AppHandle;


#[cfg(windows)]
async fn register_keybinding(app: Res<AppHandle>, injector: Injector) -> Result<(), anyhow::Error> {
    use mapp::inject::inject;
    use tauri::{async_runtime::spawn, GlobalShortcutManager};

    app.global_shortcut_manager()
        .register("Super+Alt+Q", move || {
            let injector = injector.clone();
            spawn(async move {
                let result = match inject(&injector, &interactive_window::hide_window).await {
                    Ok(v) => v,
                    Err(e) => {
                        log::debug!("inject: {}", e);
                        return;
                    }
                };

                if let Err(e) = result.await {
                    log::warn!("interactive window: {}", e);
                }
            });
        })?;
    Ok(())
}
