use clap::ArgMatches;
use itertools::Itertools;
use mapp::provider::{Injector, Res};
use mtool_interactive::{Completion, CompletionArgs, OutputDevice};

use crate::{Cmder, CommandArgs};

pub async fn exec_command_from_cli(
    args: Res<ArgMatches>,
    cmder: Res<Cmder>,
    injector: Injector,
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
                eprintln!("{} not found", cmd);
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
                    Ok(cmder
                        .get_command_fuzzy(&completed)
                        .iter()
                        .map(|c| c.get_name().to_string())
                        .collect::<Vec<_>>())
                }
            })
            .prompt("Input command..."),
        )
        .await?
    };

    {
        let command = command.clone();
        injector.construct_once(move || async move {
            let completed = c
                .complete_read(CompletionArgs::without_completion().prompt(&command))
                .await?;

            Ok(Res::new(CommandArgs::new(shellwords::split(&completed)?)))
        });
    }

    match cmder.get_command_exact(&command) {
        Some(cmd) => cmd.exec(&injector).await?,
        None => o.show_plain(&format!("{} not found", command)).await?,
    }

    injector.remove::<Res<CommandArgs>>();
    Ok(())
}
