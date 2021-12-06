use crate::core::{config::configer::GetConfig, evbus::Sender};

use anyhow::Context;
use async_trait::async_trait;
use cloud_api::tencent::{
    api,
    credential::Credential,
    translate::text::{self, TextTranslateRequest, TextTranslateResponse},
};
use serde::Deserialize;
use thiserror::Error;

use super::LanguageType;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
struct Config {
    pub credential: Credential,
}

impl From<LanguageType> for text::LanguageType {
    fn from(val: LanguageType) -> Self {
        match val {
            LanguageType::Auto => text::LanguageType::Auto,
            LanguageType::En => text::LanguageType::En,
            LanguageType::Zh => text::LanguageType::Zh,
            LanguageType::Ja => text::LanguageType::Ja,
        }
    }
}

pub struct Translator {
    cfg: Config,
}

impl Translator {
    pub async fn new(tx: &Sender) -> Result<Self> {
        Ok(Self {
            cfg: GetConfig::post(tx, "translate".into())
                .await
                .context("Get config translate")?,
        })
    }
}

#[async_trait]
impl super::Translator for Translator {
    async fn text_translate(
        &mut self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> anyhow::Result<String> {
        let req = TextTranslateRequest::new(text, source.into(), target.into());
        let resp: TextTranslateResponse = api::post(&req, &self.cfg.credential).await?;
        Ok(resp.target_text)
    }
}
