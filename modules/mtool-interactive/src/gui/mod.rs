pub mod completion;
mod interactive_window;
mod output;

pub use completion::Completion;
pub use interactive_window::*;
use mtool_gui::{Builder, GuiStage};
use mtool_system::keybinding::Keybinging;
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

async fn register_keybinding(keybinding: Res<Keybinging>) -> Result<(), anyhow::Error> {
    keybinding.define_global(
        if cfg!(windows) {
            "Super+Alt+Q"
        } else {
            "M-A-q"
        },
        interactive_window::hide_window,
    ).await?;

    Ok(())
}
