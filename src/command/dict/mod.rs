use clap::Clap;

mod hjdict;
mod mdict;

use crate::error::Result;

use self::hjdict::HJDict;
use async_trait::async_trait;

#[derive(Clap, PartialEq, Debug)]
pub enum Lang {
    #[clap(alias = "JP")]
    JP,
    #[clap(alias = "EN")]
    EN,
}

#[derive(Clap)]
pub struct DictOpt {
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

#[async_trait]
trait DictQuery {
    async fn query(&self, text: &String) -> Vec<String>;
}

trait DictCap {
    fn support_languages(&self) -> Vec<Lang>;

    fn queryable(&self, lang: &Lang) -> bool {
        self.support_languages().contains(lang)
    }
}

trait Dict: DictQuery + DictCap {}

impl DictOpt {
    async fn available_dicts(&self) -> Vec<Box<dyn Dict>> {
        vec![Box::new(HJDict {})]
    }

    pub async fn run(&self) -> Result<()> {
        for result in self
            .available_dicts()
            .await
            .iter()
            .filter(|dict| dict.queryable(&self.lang))
            .map(|dict| dict.query(&self.query))
        {
            //TODO: parallel
            result.await.iter().for_each(|item| {
                println!("{}", item);
            });
        }
        Ok(())
    }
}
