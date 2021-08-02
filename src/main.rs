mod command;
mod util;

use clap::Clap;
use command::{Opts, SubCommand};
use SubCommand::Dict;
use SubCommand::Ocr;
use SubCommand::Search;
use SubCommand::Translate;

#[tokio::main]
async fn main() {
    match Opts::parse().subcmd {
        Dict(dict) => dict.run().await,
        Translate(translate) => translate.run().await,
        Search(search) => search.run().await,
        Ocr(ocr) => ocr.run().await,
    }
}
