use itertools::Itertools;
use mapp::provider::Res;

use crate::Cmder;

#[allow(unused)]
pub async fn list_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    let output = cmder
        .list_command()
        .iter()
        .map(|cmd| {
            format!(
                "{}{}    {}",
                cmd.get_name(),
                if !cmd.get_aliases().is_empty() {
                    format!("({})", cmd.get_aliases().join(","))
                } else {
                    "".into()
                },
                cmd.get_desc()
            )
        })
        .join("\n");

    println!("{}", output);

    Ok(())
}
