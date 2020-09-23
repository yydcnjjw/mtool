use clap::Clap;
use std::str::FromStr;

pub mod jp;
#[derive(PartialEq, Debug)]
pub enum Lang {
    JP,
}

impl FromStr for Lang {
    type Err = &'static str;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "jp" | "JP" => Ok(Lang::JP),
            _ => Err("no match"),
        }
    }
}

#[derive(Clap)]
pub struct Dict {
    #[clap(required(true), index(1), about("lang"), possible_values(&["jp"]))]
    pub lang: Lang,
    #[clap(required(true), index(2), about("query"))]
    pub query: String,
}
