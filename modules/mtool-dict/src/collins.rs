use clap::Parser;
use futures::FutureExt;
use mapp::provider::Res;
use mdict::decode::collins::{self, output::OutputOrg};
use mtool_cmder::CommandArgs;
use mtool_interactive::OutputDevice;

/// Dict module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    /// query
    query: String,
}

pub async fn dict(args: Res<CommandArgs>, o: Res<OutputDevice>) -> Result<(), anyhow::Error> {
    let Args { query } = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            return Ok(());
        }
    };

    o.output_future(
        async move {
            let result: Vec<String> = match collins::dict::query(&query).await {
                Ok(result) => result
                    .iter()
                    .filter_map(|v| v.to_string_org().ok())
                    .collect(),
                Err(e) => return e.to_string(),
            };

            if result.is_empty() {
                format!("Failed to query dict {}", query)
            } else {
                result.join("\n")
            }
        }
        .boxed(),
    )
    .await?;

    Ok(())
}

pub async fn thesaures(args: Res<CommandArgs>) -> Result<(), anyhow::Error> {
    let Args { query } = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            return Ok(());
        }
    };

    let result: Vec<String> = collins::thesaures::query(&query)
        .await?
        .iter()
        .filter_map(|v| v.to_string_org().ok())
        .collect();

    if result.is_empty() {
        println!("Failed to query thesaures {}", query);
    } else {
        for item in result {
            println!("{}", item);
        }
    }

    Ok(())
}
