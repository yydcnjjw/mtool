use async_trait::async_trait;

use crate::language::LanguageType;

#[async_trait]
pub trait Translator {
    async fn text_translate(
        &mut self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> anyhow::Result<String>;
}
