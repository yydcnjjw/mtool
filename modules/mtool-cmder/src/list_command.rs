use itertools::Itertools;
use mapp::provider::Res;
use mtool_interactive::OutputDevice;

use crate::Cmder;

pub async fn list_command(cmder: Res<Cmder>, o: Res<OutputDevice>) -> Result<(), anyhow::Error> {
    let output = cmder
        .list_command()
        .iter()
        .map(|cmd| {
            format!(
                "{}{}",
                cmd.get_name(),
                if !cmd.get_aliases().is_empty() {
                    format!("({})", cmd.get_aliases().join(","))
                } else {
                    "".into()
                }
            )
        })
        .join("\n");

    o.show_plain(&output).await?;

    Ok(())
}
