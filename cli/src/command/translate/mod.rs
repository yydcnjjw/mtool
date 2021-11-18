use std::fmt::Result;

use super::{Command, Commander};
use crate::app::App;
use async_trait::async_trait;

mod tencent;

// use clap::Clap;

// use crate::error::Result;

// mod google;

#[derive(PartialEq, Debug, Clone)]
pub enum LanguageType {
    Auto,
    En,
    Zh,
    Ja,
}

// #[derive(Clap, PartialEq, Debug)]
// pub enum Backend {
//     Google,
// }

// #[derive(Clap, PartialEq, Debug)]
// pub enum Display {
//     Window,
//     Stdio,
// }

// #[derive(Clap)]
// pub struct TranslateCmd {
//     /// query
//     #[clap(required(true), index(1))]
//     query: String,
//     /// from
//     #[clap(arg_enum, default_value("en"), short, long)]
//     from: Lang,
//     /// to
//     #[clap(arg_enum, default_value("zh"), short, long)]
//     to: Lang,
//     /// backend
//     #[clap(arg_enum, default_value("google"), short, long)]
//     backend: Backend,
//     // display
//     #[clap(arg_enum, default_value("stdio"), short, long)]
//     display: Display,
// }

// impl TranslateCmd {
//     pub async fn run(&self) -> Result<()> {
//         let result = match self.backend {
//             Backend::Google => google::query(&self.query, &self.from, &self.to).await,
//         };

//         if self.display == Display::Stdio {
//             println!("{}", result.unwrap());
//         }
//         Ok(())
//     }
// }

#[async_trait]
pub trait Translator {
    async fn text_translate(
        &mut self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> anyhow::Result<String>;
}

pub struct Cmd {
    source: LanguageType,
    target: LanguageType,
    translator: tencent::Translator,
}

impl Cmd {
    pub fn new(app: &mut App, source: LanguageType, target: LanguageType) -> anyhow::Result<Self> {
        Ok(Self {
            translator: tencent::Translator::new(app)?,
            source,
            target,
        })
    }
}

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<()> {
        assert!(args.len() == 1);

        let text = args.first().unwrap();
        let result = self
            .translator
            .text_translate(text.clone(), self.source.clone(), self.target.clone())
            .await?;
        println!("{}", result);
        Ok(())
    }
}

pub fn add_command(app: &mut App) -> anyhow::Result<()> {
    {
        let cmd = Box::new(Cmd::new(app, LanguageType::Auto, LanguageType::Zh)?);
        app.cmder.insert("tz".into(), cmd);
    }

    {
        let cmd = Box::new(Cmd::new(app, LanguageType::Auto, LanguageType::En)?);
        app.cmder.insert("te".into(), cmd)
    }

    Ok(())
}
