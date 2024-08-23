use std::io::{stdin, stdout, Write};

use mapp::{prelude::*, CreateOnceTaskDescriptor};

use mtool_cmder::{Cmder, CommandArgs, CommandBuilder};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    CmdlineStage,
};

use crate::translator::{// llama,
                        openai, tencent, LanguageType, Translator};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
enum Backend {
    Tencent,
    Openai,
    // Llama,
}

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    text: Option<String>,
    #[clap(short, long, value_enum)]
    #[arg(default_value_t = Backend::Tencent)]
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
        // Backend::Llama => injector.get::<Res<llama::Translator>>().await?,
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
    }

    if let Some(text) = text {
        let result = translator
            .text_translate(text, source.clone(), target.clone())
            .await?;
        println!("{}", result);
    } else {
        println!("Please input required content");
    }

    Ok(())
}

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

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(
            CmdlineStage::AfterInit,
            register_command.cond(is_startup_mode(StartupMode::Cli)),
        );
        Ok(())
    }
}

pub async fn register_command(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(
            text_translate_into_english
                .name("translate.text.auto_into_english")
                .add_alias("te")
                .descrption("Translate text into English"),
        )
        .add_command(
            text_translate_into_chinese
                .name("translate.text.auto_into_chinese")
                .add_alias("tz")
                .descrption("Translate text into Chinese"),
        )
        .add_command(
            text_translate_into_japanese
                .name("translate.text.auto_into_japanese")
                .add_alias("tj")
                .descrption("Translate text into Japanese"),
        );
    Ok(())
}
