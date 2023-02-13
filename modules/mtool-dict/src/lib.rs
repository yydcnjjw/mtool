mod collins;
mod mdx;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mdx::mdx_query;
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::CmdlineStage;
use mtool_system::keybinding::Keybinging;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(CmdlineStage::AfterInit, register_command)
            .add_once_task(
                #[cfg(windows)]
                GuiStage::AfterInit,
                #[cfg(not(windows))]
                CmdlineStage::AfterInit,
                register_keybinding,
            );
        Ok(())
    }
}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(mdx_query.name("mdx"))
        .add_command(collins::dict.name("dict").add_alias("d"))
        .add_command(collins::thesaures.name("thesaures").add_alias("dt"));
    Ok(())
}

async fn register_keybinding(_keybinding: Res<Keybinging>) -> Result<(), anyhow::Error> {
    // keybinding
    //     .define_global(
    //         if cfg!(windows) {
    //             "Super+Alt+D"
    //         } else {
    //             "M-A-d"
    //         },
    //         collins::dict,
    //     )
    //     .await?;

    Ok(())
}
