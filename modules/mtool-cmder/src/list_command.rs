use itertools::Itertools;
use mapp::provider::Res;
use tabled::{Tabled, Table};

use crate::Cmder;

#[derive(Tabled)]
struct CommandItem {
    full_name: String,
    aliases: String,
    description: String
}


pub async fn list_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {

    let output = cmder
        .list_command()
        .iter()
        .map(|cmd| {
            CommandItem {
                full_name: cmd.get_name().to_string(),
                aliases: cmd.get_aliases().join(",").to_string(),
                description: cmd.get_desc().to_string(),
            }
        }).collect_vec();

    println!("{}", Table::new(output).to_string());

    Ok(())
}
