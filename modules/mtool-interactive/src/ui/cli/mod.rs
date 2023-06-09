use std::sync::Arc;

use crate::output::Output;
use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule, CreateOnceTaskDescriptor,
};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    CmdlineStage,
};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(
            CmdlineStage::AfterInit,
            init.cond(is_startup_mode(StartupMode::Cli)),
        );
        Ok(())
    }
}

async fn init(injector: Injector) -> Result<(), anyhow::Error> {
    injector.insert(Res::new(OutputDevice::new()));
    Ok(())
}

struct OutputDevice {}

impl OutputDevice {
    fn new() -> crate::output::OutputDevice {
        crate::output::OutputDevice(Arc::new(Self {}))
    }
}

#[async_trait]
impl Output for OutputDevice {
    async fn output(&self, s: &str) -> Result<(), anyhow::Error> {
        print!("{}", s);
        Ok(())
    }
}
