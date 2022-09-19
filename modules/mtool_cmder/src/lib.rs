mod cmder;
mod command;

use std::{
    io::{stdout, Write},
    ops::Deref,
};

use clap::{arg, ArgMatches};
pub use cmder::*;
pub use command::*;

use async_trait::async_trait;
use itertools::Itertools;
use mapp::{AppContext, AppModule, CreateTaskDescriptor, Injector, Res};

use mtool_core::{config::is_cli, Cmdline, InitStage, RunStage, StartupStage};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct(Cmder::new).await;

        app.schedule()
            .add_task(StartupStage::Startup, setup_cmdline)
            .await
            .add_task(InitStage::Init, init.cond(is_cli))
            .await
            .add_task(RunStage::Run, exec_command.cond(is_cli))
            .await;

        Ok(())
    }
}

async fn list_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    for cmd in cmder.list_command().await {
        print!("{}", cmd.get_name());

        if !cmd.get_aliases().is_empty() {
            print!("({})", cmd.get_aliases().join(","));
        }
        println!();
    }
    Ok(())
}

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline
        .setup(|cmdline| {
            Ok(cmdline.arg(arg!(<command> ... "commands to run").trailing_var_arg(true)))
        })
        .await?;

    Ok(())
}

async fn init(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(list_command.name("list_command").add_alias("lc"))
        .await;

    Ok(())
}

pub struct CommandArgs {
    inner: Vec<String>,
}

impl Deref for CommandArgs {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl CommandArgs {
    pub fn new(inner: Vec<String>) -> Self {
        Self { inner }
    }
}

async fn exec_command(
    cmder: Res<Cmder>,
    args: Res<ArgMatches>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    if let Some(cmd) = args
        .get_many::<String>("command")
        .map(|cmd| cmd.collect_vec())
    {
        let (cmd, args) = cmd.split_first().unwrap();
        match cmder.get_command_exact(cmd).await {
            Some(cmd) => {
                injector
                    .insert(Res::new(CommandArgs::new(
                        args.iter().map(|arg| arg.to_string()).collect_vec(),
                    )))
                    .await;
                cmd.exec(&injector).await?;
            }
            None => println!("{} not found", cmd),
        };
    }
    Ok(())
}
