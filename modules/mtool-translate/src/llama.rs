use anyhow::Context;
use async_trait::async_trait;
use mapp::provider::Res;
use mllama_sys::{ChatConfig, LLamaContext, LLamaContextParam};
use mtool_core::ConfigStore;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::translator::{self, LanguageType};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct ContextConfig {
    n_ctx: i32,
    n_parts: i32,
    seed: i32,
    f16_kv: bool,
    logits_all: bool,
    vocab_only: bool,
    use_mmap: bool,
    use_mlock: bool,
    embedding: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            n_ctx: 2048,
            n_parts: -1,
            seed: -1,
            f16_kv: true,
            logits_all: false,
            vocab_only: false,
            use_mmap: true,
            use_mlock: false,
            embedding: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    model: String,
    #[serde(default)]
    context: ContextConfig,
    #[serde(default)]
    chat: ChatConfig,
}

impl From<ContextConfig> for LLamaContextParam {
    fn from(c: ContextConfig) -> Self {
        Self {
            n_ctx: c.n_ctx,
            n_parts: c.n_parts,
            seed: c.seed,
            f16_kv: c.f16_kv,
            logits_all: c.logits_all,
            vocab_only: c.vocab_only,
            use_mmap: c.use_mmap,
            use_mlock: c.use_mlock,
            embedding: c.embedding,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Translator {
    chatbot: Mutex<mllama_sys::Chat>,
}

impl Translator {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs
            .get::<Config>("translate.llama")
            .await
            .context("Failed to parse translate")?;

        let ctx = LLamaContext::new(&cfg.model, LLamaContextParam::from(cfg.context))?;

        Ok(Res::new(Self {
            chatbot: Mutex::new(mllama_sys::Chat::new(ctx, cfg.chat)?),
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
        let target = match target {
            LanguageType::Auto => "English",
            LanguageType::En => "English",
            LanguageType::Zh => "Chinese",
            LanguageType::Ja => "Japanese",
        };

        let mut chatbot = self.chatbot.lock().await;

        chatbot.chat(&format!("Translate \"{text}\" into {target}"))
    }
}
