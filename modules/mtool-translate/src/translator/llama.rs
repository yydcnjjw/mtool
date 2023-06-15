use async_trait::async_trait;
use mapp::provider::Res;
use mtool_ai::LLamaChat;

use crate::translator::{self, LanguageType};

#[derive(Debug)]
pub struct Translator {
    chatbot: Res<LLamaChat>,
}

impl Translator {
    pub async fn construct(chatbot: Res<LLamaChat>) -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self { chatbot }))
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
        let target = match target {
            LanguageType::Auto => "English",
            LanguageType::En => "English",
            LanguageType::Zh => "Chinese",
            LanguageType::Ja => "Japanese",
        };
        self.chatbot
            .chat(&format!("Translate \"{text}\" into {target}"))
            .await
    }
}
