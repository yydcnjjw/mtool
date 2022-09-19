use mapp::Res;
use mtool_cmder::CommandArgs;

use crate::{language::LanguageType, tencent, translator::Translator};

use clap::Parser;

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    text: String,
}

async fn text_translate(
    source: LanguageType,
    target: LanguageType,
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            return Ok(());
        }
    };

    let Args { text } = args;

    let result = translator.text_translate(text, source, target).await?;
    println!("{}", result);

    Ok(())
}

pub async fn tz(
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
) -> Result<(), anyhow::Error> {
    text_translate(LanguageType::Auto, LanguageType::Zh, args, translator).await
}

pub async fn te(
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
) -> Result<(), anyhow::Error> {
    text_translate(LanguageType::Auto, LanguageType::En, args, translator).await
}

pub async fn tj(
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
) -> Result<(), anyhow::Error> {
    text_translate(LanguageType::Auto, LanguageType::Ja, args, translator).await
}
