use clap::Parser;
use mapp::Res;
use mdict::decode::collins::{self, output::OutputOrg};
use mtool_cmder::CommandArgs;

/// Dict module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    /// query
    query: String,
}

pub async fn dict(args: Res<CommandArgs>) -> Result<(), anyhow::Error> {
    let Args { query } = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            return Ok(());
        }
    };

    let result: Vec<String> = collins::dict::query(&query)
        .await?
        .iter()
        .filter_map(|v| v.to_string_org().ok())
        .collect();

    if result.is_empty() {
        println!("Failed to query dict {}", query);
    } else {
        for item in result {
            println!("{}", item);
        }
    }

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
