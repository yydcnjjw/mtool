use clap::Clap;
use dict::Dict;

pub mod dict;

#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    // dict
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Dict(Dict),
}
