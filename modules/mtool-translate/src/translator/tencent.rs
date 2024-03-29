use anyhow::Context;
use async_trait::async_trait;
use mapp::provider::Res;
use mcloud_api::tencent::{
    api,
    credential::Credential,
    translate::text::{self, TextTranslateRequest, TextTranslateResponse},
};
use mtool_core::ConfigStore;
use serde::Deserialize;

use crate::translator::{self, LanguageType};

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(flatten)]
    credential: Credential,
}

#[derive(Debug)]
pub struct Translator {
    cfg: Config,
}

impl Translator {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs
            .get::<Config>("translate.tencent")
            .await
            .context("Failed to parse translate")?;

        Ok(Res::new(Self { cfg }))
    }
}

#[async_trait]
impl translator::Translator for Translator {
    async fn text_translate(
        &self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> Result<String, anyhow::Error> {
        let req = TextTranslateRequest::new(text, source.into(), target.into());
        let resp: TextTranslateResponse = api::post(&req, &self.cfg.credential)
            .await
            .context(format!("Failed to request tencent cloud api: {:?}", req))?;
        Ok(resp.target_text)
    }
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
