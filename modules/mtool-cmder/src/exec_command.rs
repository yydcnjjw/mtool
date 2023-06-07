use std::ops::Deref;

use anyhow::Context;
use clap::ArgMatches;
use itertools::Itertools;
use mapp::provider::{Injector, Res, Take};
use mtool_interactive::{CompleteItem, Completion, CompletionArgs, Props, TryFromCompleted};
use yew::prelude::*;

use crate::{Cmder, CommandArgs, SharedCommandDescriptor};

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
                injector.insert(Take::new(CommandArgs::new(
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

#[derive(Properties, PartialEq, Clone)]
struct CommandItem {
    inner: SharedCommandDescriptor,
}

impl Deref for CommandItem {
    type Target = SharedCommandDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromCompleted for CommandItem {}

impl CompleteItem for CommandItem {
    type WGuiView = CommandItemView;

    fn complete_hint(&self) -> String {
        self.get_name().to_string()
    }
}

#[function_component]
fn CommandItemView(props: &Props<CommandItem>) -> Html {
    html! {
        <div> { props.get_name() } </div>
    }
}

pub async fn exec_command_interactive(
    c: Res<Completion>,
    cmder: Res<Cmder>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    let command = {
        let cmder = cmder.clone();
        let command = c
            .complete_read(
                CompletionArgs::with_vec(
                    cmder
                        .list_command()
                        .into_iter()
                        .map(|v| CommandItem { inner: v })
                        .collect_vec(),
                )
                .prompt("Input command..."),
            )
            .await?;
        match command {
            Some(command) => command,
            None => return Ok(()),
        }
    };

    {
        let command = command.clone();
        injector.construct_once(move || async move {
            let completed = c
                .complete_read(
                    CompletionArgs::<String>::without_completion().prompt(command.get_name()),
                )
                .await?
                .context("complete read canceled")?;
            Ok(Take::new(CommandArgs::new(shellwords::split(&completed)?)))
        });
    }

    command.exec(&injector).await?;
    Ok(())
}
