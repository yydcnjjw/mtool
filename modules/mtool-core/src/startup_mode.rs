use mapp::{
    anyhow::{self, anyhow},
    futures::{future::BoxFuture, FutureExt},
    prelude::*,
};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            use crate::{Cmdline, CmdlineStage};
            use clap::{arg, ArgMatches};

            async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
                cmdline.setup(|cmdline| {
                    Ok(cmdline.arg(
                        arg!(--mode <MODE> "startup mode")
                            .value_parser(["cli", "gui"])
                            .default_value("cli"),
                    ))
                })?;

                Ok(())
            }
            app.schedule()
                .add_once_task(CmdlineStage::Setup, setup_cmdline);

            async fn startup_mode(
                args: Res<ArgMatches>,
            ) -> Result<Res<StartupMode>, anyhow::Error> {
                Ok(Res::new(StartupMode::from(
                    args.get_one::<String>("mode")
                        .ok_or(anyhow!("missing mode"))?
                        .as_str(),
                )))
            }
            app.injector().construct_once(startup_mode);
        }

        #[cfg(target_os = "android")]
        {
            app.injector().insert(Res::new(StartupMode::Gui));
        }

        Ok(())
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum StartupMode {
    Gui,
    Cli,
}

impl From<&str> for StartupMode {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "gui" => StartupMode::Gui,
            "cli" => StartupMode::Cli,
            _ => unreachable!(),
        }
    }
}

pub fn run_on_gui(
) -> impl Fn(Res<StartupMode>) -> BoxFuture<'static, Result<bool, anyhow::Error>> + Clone {
    move |mode: Res<StartupMode>| async move { Ok(*mode == StartupMode::Gui) }.boxed()
}

pub fn run_on_cli(
) -> impl Fn(Res<StartupMode>) -> BoxFuture<'static, Result<bool, anyhow::Error>> + Clone {
    move |mode: Res<StartupMode>| async move { Ok(*mode == StartupMode::Cli) }.boxed()
}
