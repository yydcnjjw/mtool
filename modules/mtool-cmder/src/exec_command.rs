use anyhow::Context;
use clap::{arg, ArgMatches};
use itertools::Itertools;
use mapp::provider::{Injector, Res};
use mtool_interactive::{Completion, CompletionArgs, OutputDevice};

use crate::{Cmder, CommandArgs};

pub async fn exec_command(
    args: Res<ArgMatches>,
    cmder: Res<Cmder>,
    injector: Injector,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    if let Some(cmd) = args
        .get_many::<String>("command")
        .map(|cmd| cmd.collect_vec())
    {
        let (cmd, args) = cmd.split_first().unwrap();

        match cmder.get_command_exact(cmd) {
            Some(cmd) => {
                injector.insert(Res::new(CommandArgs::new(
                    args.iter().map(|arg| arg.to_string()).collect_vec(),
                )));
                cmd.exec(&injector).await?;
            }
            None => {
                o.show_plain(&format!("{} not found", cmd)).await?;
            }
        };
    }
    Ok(())
}

pub async fn exec_command_interactive(
    c: Res<Completion>,
    cmder: Res<Cmder>,
    injector: Injector,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    let command = {
        let cmder = cmder.clone();
        c.complete_read(
            CompletionArgs::new(move |completed: String| {
                let cmder = cmder.clone();
                async move {
                    Ok::<Vec<String>, anyhow::Error>(
                        cmder
                            .get_command_fuzzy(&completed)
                            .iter()
                            .map(|c| c.get_name().to_string())
                            .collect::<Vec<_>>(),
                    )
                }
            })
            .prompt(">"),
        )
        .await?
    };

    let args = c
        .complete_read(CompletionArgs::new(|_| async move { Ok(Vec::new()) }).prompt(&command))
        .await?;

    let args = clap::command!()
        .arg(arg!([command] ... "commands to run").trailing_var_arg(true))
        .no_binary_name(true)
        .try_get_matches_from(
            shellwords::split(&format!("{} {}", command, args))
                .context("Failed to split command line args")?,
        )
        .context("Failed to parse command line args")?;

    exec_command(Res::new(args), cmder, injector, o).await
}
