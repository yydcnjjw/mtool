use clap::Clap;

#[derive(Clap, PartialEq, Debug)]
pub enum Lang {
    en,
    zh,
}

#[derive(Clap)]
pub struct Translate {
    /// query
    #[clap(required(true), index(1))]
    query: String,
    /// from
    #[clap(arg_enum, default_value("en"), short, long)]
    from: Lang,
    /// to
    #[clap(arg_enum, default_value("zh"), short, long)]
    to: Lang,
}
