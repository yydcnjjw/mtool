use std::io::{stdin, stdout, Write};

use futures::FutureExt;
use mapp::provider::{Injector, Res, Take};
use mtool_cmder::CommandArgs;
use mtool_interactive::{Completion, CompletionArgs, OutputDevice};

use crate::{
    llama, openai, tencent,
    translator::{LanguageType, Translator},
};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
enum Backend {
    Tencent,
    Openai,
    Llama,
}

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    text: Option<String>,
    #[clap(short, long, value_enum)]
    #[arg(default_value_t = Backend::Openai)]
    backend: Backend,
    #[clap(short, long)]
    interactive: bool,
}

async fn text_translate_from_cli(
    source: LanguageType,
    target: LanguageType,
    args: Take<CommandArgs>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.take()?.iter()) {
        Ok(args) => args,
        Err(e) => {
            e.print()?;
            return Ok(());
        }
    };

    let Args {
        text,
        backend,
        interactive,
    } = args;

    let translator: Res<dyn Translator + Send + Sync> = match backend {
        Backend::Tencent => injector.get::<Res<tencent::Translator>>().await?,
        Backend::Openai => injector.get::<Res<openai::Translator>>().await?,
        Backend::Llama => injector.get::<Res<llama::Translator>>().await?,
    };

    if interactive {
        loop {
            print!("> ");
            stdout().flush()?;
            let input = tokio::task::spawn_blocking(|| {
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                input
            })
            .await?;

            let result = translator
                .text_translate(input, source.clone(), target.clone())
                .await?;
            println!("{}", result);
        }
    } else if let Some(text) = text {
        let result = translator
            .text_translate(text, source.clone(), target.clone())
            .await?;
        println!("{}", result);
    } else {
        println!("Please input required content");
    }

    Ok(())
}

async fn text_translate_wgui(
    source: LanguageType,
    target: LanguageType,
    translator: Res<llama::Translator>,
    c: Res<Completion>,
    o: Res<OutputDevice>,
) -> Result<(), anyhow::Error> {
    let text = match c
        .complete_read(CompletionArgs::without_completion().prompt("Input text: "))
        .await?
    {
        Some(v) => v,
        None => return Ok(()),
    };

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

macro_rules! quick_translate_wgui {
    ($name:ident, $source:ident, $target:ident) => {
        #[allow(unused)]
        pub async fn $name(
            translator: Res<llama::Translator>,
            c: Res<Completion>,
            o: Res<OutputDevice>,
        ) -> Result<(), anyhow::Error> {
            text_translate_wgui(
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

quick_translate_wgui!(text_translate_into_chinese_wgui, Auto, Zh);
quick_translate_wgui!(text_translate_into_english_wgui, Auto, En);
quick_translate_wgui!(text_translate_into_japanese_wgui, Auto, Ja);

macro_rules! quick_translate {
    ($name:ident, $source:ident, $target:ident) => {
        pub async fn $name(
            args: Take<CommandArgs>,
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
