mod command;
mod opts;
mod util;
mod config;

use clap::Clap;
use command::SubCommand;
use opts::Opts;
use SubCommand::*;

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    
    match opts.subcmd {
        Dict(dict) => dict.run().await,
        Translate(translate) => translate.run().await,
        Search(search) => search.run().await,
        Ocr(ocr) => ocr.run().await,
        Mdict(mdict) => mdict.run().await,
    }
}
