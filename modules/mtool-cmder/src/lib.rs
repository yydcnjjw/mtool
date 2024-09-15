#[cfg(not(target_family = "wasm"))]
mod cmd;
mod cmder;
mod command;
mod command_args;
mod fn_command;

pub use cmder::*;
pub use command::*;
pub use command_args::*;

use mapp::{anyhow, prelude::*};

#[derive(Default)]
pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, app: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(LocalCmder::construct);
        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        use clap::arg;
        use mapp::CreateOnceTaskDescriptor;
        use mtool_core::{
            config::{is_startup_mode, StartupMode},
            AppStage, Cmdline, CmdlineStage,
        };

        app.injector().construct_once(Cmder::construct);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(
                CmdlineStage::AfterInit,
                register_command.cond(is_startup_mode(StartupMode::Cli)),
            )
            .add_once_task(
                AppStage::Run,
                cmd::exec_command.cond(is_startup_mode(StartupMode::Cli)),
            );

        async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
            cmdline.setup(|cmdline| {
                Ok(cmdline.arg(arg!([command] ... "commands to run").trailing_var_arg(true)))
            })?;

            Ok(())
        }

        async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
            cmder.add_command(
                cmd::list_command
                    .name("list_command")
                    .add_alias("lc")
                    .descrption("List commands"),
            );
            Ok(())
        }

        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-cmder");
    group.add_module(Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-cmder");
    group.add_module(Module);
    group
}
