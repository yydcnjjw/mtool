pub mod dict;
pub mod search;
pub mod translate;

use crate::command::dict::Dict;
use crate::command::search::Search;
use crate::command::translate::Translate;

use clap::Clap;

/// my tool
#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    /// dict
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Dict(Dict),
    /// translate
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Translate(Translate),
    /// search
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Search(Search),
}
