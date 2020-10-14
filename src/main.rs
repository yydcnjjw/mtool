mod command;
mod util;

use clap::Clap;
use command::{Opts, SubCommand};
use gio::prelude::*;
use gtk::prelude::*;
#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Dict(dict) => dict.do_query().await,
        SubCommand::Translate(translate) => translate.do_query().await,
    }
}
