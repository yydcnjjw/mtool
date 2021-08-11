use clap::{AppSettings, Clap};

use crate::command::SubCommand;

/// my tool
#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
    /// config path
    #[clap(short, long, default_value = "/home/yydcnjjw/.my-tool/config.toml")]
    pub config_path: String,
}
