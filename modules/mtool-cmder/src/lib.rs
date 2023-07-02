mod cmder;
mod command;
mod command_args;
mod exec_command;
mod list_command;

pub use cmder::*;
pub use command::*;
pub use command_args::*;

#[allow(unused_imports)]
use exec_command::*;
#[allow(unused_imports)]
use list_command::*;

use async_trait::async_trait;
use mapp::prelude::*;

#[derive(Default)]
pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, app: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        use mtool_wgui::Templator;
        use mtool_wgui::WebStage;

        app.schedule()
            .add_once_task(WebStage::Init, |templator: Res<Templator>| async move {
                templator.add_template::<CommandItemView>();
                Ok::<(), anyhow::Error>(())
            });

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
            config::{is_startup_mode, not_startup_mode, StartupMode},
            AppStage, Cmdline, CmdlineStage,
        };
        use mtool_system::keybinding::Keybinding;

        app.injector().construct_once(Cmder::new);

        app.schedule()
            .add_once_task(CmdlineStage::Setup, setup_cmdline)
            .add_once_task(
                CmdlineStage::AfterInit,
                register_command.cond(is_startup_mode(StartupMode::Cli)),
            )
            .add_once_task(
                AppStage::Init,
                register_keybinding.cond(not_startup_mode(StartupMode::Cli)),
            )
            .add_once_task(
                AppStage::Run,
                exec_command_from_cli.cond(is_startup_mode(StartupMode::Cli)),
            );

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

        async fn register_keybinding(keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
            keybinding
                .define_global("M-A-x", exec_command_interactive)
                .await?;
            Ok(())
        }

        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-cmder");
    group.add_module(Module);
    return group;
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-cmder");
    group.add_module(Module);
    return group;
}
