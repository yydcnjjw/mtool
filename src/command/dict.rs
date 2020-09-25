use clap::Clap;

pub mod jp;

#[derive(Clap, PartialEq, Debug)]
pub enum Lang {
    #[clap(alias = "JP")]
    JP,
}

#[derive(Clap)]
pub struct Dict {
    /// lang
    #[clap(arg_enum, required(true), index(1))]
    lang: Lang,
    /// query
    #[clap(required(true), index(2))]
    query: String,
    /// save to anki
    #[clap(short, long)]
    save: bool,
}

impl Dict {
    pub async fn do_query(&self) {
        match self.lang {
            Lang::JP => match jp::query(&self.query).await {
                Ok(word) => {
                    println!("{}", &word.to_cli_str());
                    if self.save {
                        match jp::save(&word).await {
                            Ok(_) => {}
                            Err(e) => println!("{}", e),
                        }
                    }
                }
                Err(e) => println!("{}", e),
            },
        }
    }
}
