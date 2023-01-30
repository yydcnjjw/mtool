use mapp::provider::Res;
use mtool_cmder::CommandArgs;
use mtool_interactive::OutputDevice;

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
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            o.show_plain(&e.render().to_string()).await?;
            return Ok(());
        }
    };

    let Args { text } = args;

    let result = translator.text_translate(text, source, target).await?;
    o.show_plain(&result).await?;

    Ok(())
}

pub async fn tz(
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    text_translate(LanguageType::Auto, LanguageType::Zh, args, translator, o).await
}

pub async fn te(
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    text_translate(LanguageType::Auto, LanguageType::En, args, translator, o).await
}

pub async fn tj(
    args: Res<CommandArgs>,
    translator: Res<tencent::Translator>,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    text_translate(LanguageType::Auto, LanguageType::Ja, args, translator, o).await
}
