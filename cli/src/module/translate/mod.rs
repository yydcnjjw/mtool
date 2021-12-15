use crate::{
    app::App,
    core::{
        command::{self, AddCommand, Command},
        evbus::Sender,
    },
};
use anyhow::Context;
use async_trait::async_trait;
use clap::Parser;

mod tencent;

#[derive(PartialEq, Debug, Clone)]
pub enum LanguageType {
    Auto,
    En,
    Zh,
    Ja,
}

#[async_trait]
trait Translator {
    async fn text_translate(
        &mut self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> anyhow::Result<String>;
}

struct Cmd {
    source: LanguageType,
    target: LanguageType,
    translator: tencent::Translator,
}

impl Cmd {
    async fn new(tx: &Sender, source: LanguageType, target: LanguageType) -> anyhow::Result<Self> {
        Ok(Self {
            translator: tencent::Translator::new(tx).await?,
            source,
            target,
        })
    }

    async fn text_translate(&mut self, text: String) -> anyhow::Result<String> {
        let result = self
            .translator
            .text_translate(text, self.source.clone(), self.target.clone())
            .await?;
        Ok(result)
    }
}

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    text: String,
}

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<command::Output> {
        let args = Args::try_parse_from(args).context("Translate parse args failed")?;

        Ok(command::Output::String(
            self.text_translate(args.text.clone()).await?,
        ))
    }
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = &app.evbus.sender();
    AddCommand::post(
        sender,
        "tz".into(),
        Cmd::new(sender, LanguageType::Auto, LanguageType::Zh).await?,
    )
    .await?;
    AddCommand::post(
        sender,
        "te".into(),
        Cmd::new(sender, LanguageType::Auto, LanguageType::En).await?,
    )
    .await?;
    Ok(())
}
