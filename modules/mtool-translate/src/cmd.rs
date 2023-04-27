use futures::FutureExt;
use mapp::provider::{Injector, Res};
use mtool_cmder::CommandArgs;
use mtool_interactive::{Completion, CompletionArgs, OutputDevice};

use crate::{
    openai, tencent,
    translator::{LanguageType, Translator},
};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
enum Backend {
    Tencent,
    Openai,
}

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    text: String,
    #[clap(value_enum)]
    #[arg(default_value_t = Backend::Openai)]
    backend: Backend,
}

async fn text_translate_from_cli(
    source: LanguageType,
    target: LanguageType,
    args: Res<CommandArgs>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print()?;
            return Ok(());
        }
    };

    let Args { text, backend } = args;

    let translator: Res<dyn Translator + Send + Sync> = match backend {
        Backend::Tencent => injector.get::<Res<tencent::Translator>>().await?,
        Backend::Openai => injector.get::<Res<openai::Translator>>().await?,
    };

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

    o.output_future(
        async move {
            match translator.text_translate(text, source, target).await {
                Ok(o) => o,
                Err(e) => e.to_string(),
            }
        }
        .boxed(),
    )
    .await?;

    Ok(())
}

macro_rules! quick_translate_interactive {
    ($name:ident, $source:ident, $target:ident) => {
        #[allow(unused)]
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
            args: Res<CommandArgs>,
            injector: Injector,
        ) -> Result<(), anyhow::Error> {
            text_translate_from_cli(LanguageType::$source, LanguageType::$target, args, injector)
                .await
        }
    };
}

quick_translate!(text_translate_into_chinese, Auto, Zh);
quick_translate!(text_translate_into_english, Auto, En);
quick_translate!(text_translate_into_japanese, Auto, Ja);
