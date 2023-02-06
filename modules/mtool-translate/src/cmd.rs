use mapp::provider::Res;
use mtool_cmder::CommandArgs;
use mtool_interactive::{Completion, CompletionArgs, OutputDevice};

use crate::{language::LanguageType, tencent, translator::Translator};

use clap::Parser;

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    text: String,
}

async fn text_translate_from_cli(
    source: LanguageType,
    target: LanguageType,
    translator: Res<tencent::Translator>,
    args: Res<CommandArgs>,
) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print()?;
            return Ok(());
        }
    };

    let Args { text } = args;

    let result = translator.text_translate(text, source, target).await?;

    println!("{}", result);

    Ok(())
}

async fn text_translate_interactive(
    source: LanguageType,
    target: LanguageType,
    translator: Res<tencent::Translator>,
    c: Res<Completion>,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    let text = c
        .complete_read(CompletionArgs::without_completion().prompt("Input text: "))
        .await?;

    let result = translator.text_translate(text, source, target).await?;

    o.output(&result).await?;

    Ok(())
}

macro_rules! quick_translate_interactive {
    ($name:ident, $source:ident, $target:ident) => {
        pub async fn $name(
            translator: Res<tencent::Translator>,
            c: Res<Completion>,
            o: Res<OutputDevice>,
        ) -> Result<(), anyhow::Error> {
            text_translate_interactive(
                LanguageType::$source,
                LanguageType::$target,
                translator,
                c,
                o,
            )
            .await
        }
    };
}

quick_translate_interactive!(tz_interactive, Auto, Zh);
quick_translate_interactive!(te_interactive, Auto, En);
quick_translate_interactive!(tj_interactive, Auto, Ja);

macro_rules! quick_translate {
    ($name:ident, $source:ident, $target:ident) => {
        pub async fn $name(
            translator: Res<tencent::Translator>,
            args: Res<CommandArgs>,
        ) -> Result<(), anyhow::Error> {
            text_translate_from_cli(
                LanguageType::$source,
                LanguageType::$target,
                translator,
                args,
            )
            .await
        }
    };
}

quick_translate!(tz, Auto, Zh);
quick_translate!(te, Auto, En);
quick_translate!(tj, Auto, Ja);
