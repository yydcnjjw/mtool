mod cmder;
mod command;
mod command_args;
mod exec_command;
mod list_command;

pub use cmder::*;
pub use command::*;
pub use command_args::*;
use exec_command::*;
use list_command::*;

use async_trait::async_trait;
use clap::arg;
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor};

use mtool_core::{
    config::{is_startup_mode, not_startup_mode, StartupMode},
    AppStage, Cmdline, CmdlineStage,
};
#[allow(unused)]
use mtool_gui::GuiStage;
use mtool_system::keybinding::Keybinging;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Cmder::new);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(CmdlineStage::AfterInit, register_command)
            .add_once_task(
                #[cfg(windows)]
                GuiStage::AfterInit,
                #[cfg(not(windows))]
                CmdlineStage::AfterInit,
                register_keybinding.cond(not_startup_mode(StartupMode::Cli)),
            )
            .add_once_task(
                AppStage::Run,
                exec_command_from_cli.cond(is_startup_mode(StartupMode::Cli)),
            );

        Ok(())
    }
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline.setup(|cmdline| {
        Ok(cmdline.arg(arg!([command] ... "commands to run").trailing_var_arg(true)))
    })?;

    Ok(())
}

async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(list_command.name("list_command").add_alias("lc"));
    Ok(())
}

async fn register_keybinding(keybinding: Res<Keybinging>) -> Result<(), anyhow::Error> {
    keybinding.define_global(
        if cfg!(windows) {
            "Super+Alt+X"
        } else {
            "M-A-x"
        },
        exec_command_interactive,
    ).await?;

    Ok(())
}
