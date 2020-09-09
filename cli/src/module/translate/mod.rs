use crate::{app::App, core::command::Command};
use async_trait::async_trait;

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
    fn new(app: &mut App, source: LanguageType, target: LanguageType) -> anyhow::Result<Self> {
        Ok(Self {
            translator: tencent::Translator::new(app)?,
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

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<()> {
        if args.len() == 1 {
            let text = args.first().unwrap();
            log::info!("{}", self.text_translate(text.clone()).await?);
        } else {
            
        }

        Ok(())
    }
}

fn add_command(app: &mut App) -> anyhow::Result<()> {
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

pub async fn module_load(app: &mut App) -> anyhow::Result<()> {
    add_command(app)
}
