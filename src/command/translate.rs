use clap::Clap;

mod google;

#[derive(Clap, PartialEq, Debug)]
pub enum Lang {
    #[clap(alias = "AUTO")]
    AUTO,
    #[clap(alias = "EN")]
    EN,
    #[clap(alias = "ZH")]
    ZH,
}

#[derive(Clap, PartialEq, Debug)]
pub enum Backend {
    #[clap(alias = "GOOGLE")]
    GOOGLE,
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
    /// backend
    #[clap(arg_enum, default_value("google"), short, long)]
    backend: Backend,
}

impl Translate {
    pub async fn do_query(&self) {
        match self.backend {
            Backend::GOOGLE => google::query(&self.query, &self.from, &self.to).await,
        }
    }
}
