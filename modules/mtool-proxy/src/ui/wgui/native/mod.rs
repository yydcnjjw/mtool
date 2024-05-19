mod plugin;

use mtool_wgui::{Builder, WGuiStage};

use mapp::{prelude::*, CreateOnceTaskDescriptor};

use crate::service::{is_runnable, ProxyService};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WGuiStage::Setup, setup.cond(is_runnable));
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, service: Res<ProxyService>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(plugin::init(service))))?;
    Ok(())
}
