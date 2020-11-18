mod command;
mod util;

use SubCommand::Dict;
use SubCommand::Translate;
use clap::Clap;
use command::{Opts, SubCommand};
use gio::prelude::*;
use gtk::prelude::*;

#[tokio::main]
async fn main() {
    match Opts::parse().subcmd {
        Dict(dict) => dict.do_query().await,
        Translate(translate) => translate.do_query().await,
    }
}
