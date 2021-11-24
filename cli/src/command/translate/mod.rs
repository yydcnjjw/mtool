use super::Command;
use crate::app::App;
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
