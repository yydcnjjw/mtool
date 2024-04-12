use clap::ArgMatches;
use itertools::Itertools;
use mapp::prelude::*;
use tabled::{Table, Tabled};

use crate::{Cmder, CommandArgs};

#[derive(Tabled)]
struct CommandItem {
    full_name: String,
    aliases: String,
    description: String,
}

#[allow(unused)]
pub async fn list_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    let output = cmder
        .iter()
        .map(|cmd| CommandItem {
            full_name: cmd.get_name().to_string(),
            aliases: cmd.get_aliases().join(",").to_string(),
            description: cmd.get_desc().to_string(),
        })
        .collect_vec();

    println!("{}", Table::new(output).to_string());

    Ok(())
}

#[allow(unused)]
pub async fn exec_command(
    args: Res<ArgMatches>,
    cmder: Res<Cmder>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    if let Some(cmd) = args
        .get_many::<String>("command")
        .map(|cmd| cmd.collect_vec())
    {
        let (cmd, args) = cmd.split_first().unwrap();

        match cmder.get_command_with_name_or_alias(cmd) {
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
