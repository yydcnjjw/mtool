cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        pub mod llama;
        pub mod openai;
        pub mod tencent;
        use mapp::prelude::*;
    }
}

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum LanguageType {
    Auto,
    En,
    Zh,
    Ja,
}

impl fmt::Display for LanguageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageType::Auto => write!(f, "auto"),
            LanguageType::En => write!(f, "en"),
            LanguageType::Zh => write!(f, "zh"),
            LanguageType::Ja => write!(f, "ja"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    Tencent,
    Openai,
    Llama,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Tencent => write!(f, "tencent"),
            Backend::Openai => write!(f, "openai"),
            Backend::Llama => write!(f, "llama"),
        }
    }
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

#[cfg(not(target_family = "wasm"))]
pub struct Module;

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, #[allow(unused)] app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector()
            .construct_once(tencent::Translator::construct)
            .construct_once(openai::Translator::construct)
            .construct_once(llama::Translator::construct);
        Ok(())
    }
}
