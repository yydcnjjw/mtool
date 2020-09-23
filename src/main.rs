mod command;
mod util;
use clap::Clap;
use command::{Opts, SubCommand};

// my tool

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Dict(dict) => {
            if dict.lang == command::dict::Lang::JP {
                match command::dict::jp::query(&dict.query).await {
                    Ok(word) => println!("{}", &word.to_cli_str()),
                    Err(e) => println!("{}", e),
                }
            }
        }
    }
}
