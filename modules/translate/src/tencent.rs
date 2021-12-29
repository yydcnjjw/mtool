use anyhow::Context;
use async_trait::async_trait;
use mcloud_api::tencent::{
    api,
    credential::Credential,
    translate::text::{self, TextTranslateRequest, TextTranslateResponse},
};

use crate::{language::LanguageType, translator};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
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
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl translator::Translator for Translator {
    async fn text_translate(
        &mut self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> anyhow::Result<String> {
        let req = TextTranslateRequest::new(text, source.into(), target.into());
        let resp: TextTranslateResponse = api::post(&req, &self.cfg.credential)
            .await
            .context(format!("Failed to request tencent cloud api: {:?}", req))?;
        Ok(resp.target_text)
    }
}
