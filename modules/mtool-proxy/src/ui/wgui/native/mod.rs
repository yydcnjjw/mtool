mod cmd;
mod plugin;

use mtool_cmder::{Cmder, CommandBuilder};
use mtool_core::AppStage;
use mtool_wgui::{Builder, WGuiStage};

use mapp::{prelude::*, CreateOnceTaskDescriptor};

use crate::service::{is_runnable, ProxyService};

use self::cmd::show_stats;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(WGuiStage::Setup, setup_wgui.cond(is_runnable))
            .add_once_task(AppStage::Init, setup.cond(is_runnable));
        Ok(())
    }
}

async fn setup_wgui(
    builder: Res<Builder>,
    service: Res<ProxyService>,
) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(plugin::init(service))))?;
    Ok(())
}

async fn setup(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(
        show_stats
            .name("proxy.show_stats")
            .descrption("Show proxy stats"),
    );
    Ok(())
}
