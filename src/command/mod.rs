pub mod dict;
pub mod ocr;
pub mod search;
pub mod translate;

use self::{dict::Dict, ocr::Ocr, search::Search, translate::Translate};

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
    /// ocr
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Ocr(Ocr),
}
