use async_trait::async_trait;

#[derive(PartialEq, Debug, Clone)]
pub enum LanguageType {
    Auto,
    En,
    Zh,
    Ja,
}

#[async_trait]
pub trait Translator {
    async fn text_translate(
        &self,
        text: String,
        source: LanguageType,
        target: LanguageType,
    ) -> Result<String, anyhow::Error>;
}
