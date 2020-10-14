use clap::Clap;

mod google;

#[derive(Clap, PartialEq, Debug)]
pub enum Lang {
    AUTO,
    EN,
    ZH,
}

#[derive(Clap, PartialEq, Debug)]
pub enum Backend {
    Google,
}

#[derive(Clap, PartialEq, Debug)]
pub enum Display {
    Window,
    Stdio,
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
    // display
    #[clap(arg_enum, default_value("stdio"), short, long)]
    display: Display,
}

impl Translate {
    pub async fn do_query(&self) {
        let result = match self.backend {
            Backend::Google => google::query(&self.query, &self.from, &self.to).await,
        };

        if self.display == Display::Stdio {
            println!("{}", result.unwrap());
        } else {

        };
    }
}
