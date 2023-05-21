use anyhow::Context;
use async_trait::async_trait;
use itertools::Itertools;
use mapp::provider::Res;
use mcloud_api::openai::{
    chat::{ChatMessage, ChatRequest, ChatResponse},
    Client,
};
use mtool_core::ConfigStore;
use serde::Deserialize;

use crate::translator::{self, LanguageType};

#[derive(Debug, Clone, Deserialize)]
struct Config {
    key: String,
}

#[derive(Debug)]
pub struct Translator {
    cli: Client,
}

impl Translator {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs
            .get::<Config>("translate.openai")
            .await
            .context("Failed to parse translate")?;

        Ok(Res::new(Self {
            cli: Client::new(&cfg.key)?,
        }))
    }
}

#[async_trait]
impl translator::Translator for Translator {
    async fn text_translate(
        &self,
        text: String,
        _source: LanguageType,
        target: LanguageType,
    ) -> Result<String, anyhow::Error> {
        let mut req = ChatRequest::default();

        let target = match target {
            LanguageType::Auto => "English",
            LanguageType::En => "English",
            LanguageType::Zh => "Chinese",
            LanguageType::Ja => "Japanese",
        };

        req.messages = vec![ChatMessage {
            role: "user".into(),
            content: format!("I want you to act as an {target} translator, spelling corrector and improver. I will speak to you in any language and you will detect the language, translate it and answer in the corrected and improved version of my text, in {target}. I want you to replace my simplified A0-level words and sentences with more beautiful and elegant, upper level {target} words and sentences. Keep the meaning same, but make them more literary. I want you to only reply the correction, the improvements and nothing else, do not write explanations. My first sentence is \"{text}\"").into(),
            ..Default::default()
        }];

        let resp: ChatResponse = self.cli.send(&req).await?;
        Ok(resp
            .choices
            .into_iter()
            .map(|c| c.message.content)
            .collect_vec()
            .concat())
    }
}
