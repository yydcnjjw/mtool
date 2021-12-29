use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    language::LanguageType,
    tencent::{self, Config},
    translator::Translator,
};

use anyhow::Context;
use clap::Parser;
use cmder_mod::Command;
use tokio::sync::Mutex;

/// Translate module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    text: String,
}

pub struct Cmd {
    source: LanguageType,
    target: LanguageType,
    translator: tencent::Translator,
}

impl Cmd {
    pub fn new(cfg: Config, source: LanguageType, target: LanguageType) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            translator: tencent::Translator::new(cfg),
            source,
            target,
        }))
    }

    async fn text_translate(&mut self, text: String) -> anyhow::Result<String> {
        self.translator
            .text_translate(text, self.source.clone(), self.target.clone())
            .await
    }
}

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, args: Vec<String>) {
        let args = Args::try_parse_from(args).context("Failed to parse translate args");
        if let Err(ref e) = args {
            log::warn!("{:?}", e);
            return;
        }

        let Args { text } = args.unwrap();

        match self.text_translate(text).await {
            Ok(result) => {
                println!("{}", result);
            }
            Err(e) => {
                log::warn!("{:?}", e);
            }
        }
    }
}
