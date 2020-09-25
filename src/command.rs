use clap::Clap;
use dict::Dict;
use translate::Translate;

pub mod dict;
pub mod translate;

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
}
