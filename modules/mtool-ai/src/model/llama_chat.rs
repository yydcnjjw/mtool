use anyhow::Context;
use mapp::provider::Res;
use mllama_sys::{Chat, ChatConfig, LLamaContext, LLamaContextParam};
use mtool_core::ConfigStore;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

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

#[derive(Debug, Clone, Deserialize)]
struct Config {
    model: String,
    #[serde(default)]
    context: ContextConfig,
    #[serde(default)]
    chat: ChatConfig,
}

#[derive(Debug)]
pub struct LLamaChat {
    tx: mpsc::UnboundedSender<(String, oneshot::Sender<String>)>,
}

impl LLamaChat {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs
            .get::<Config>("ai.llama")
            .await
            .context("Failed to parse ai.llama")?;

        Ok(Res::new(Self::new(cfg)?))
    }

    pub async fn chat(&self, text: &str) -> Result<String, anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send((text.to_string(), tx))
            .context("send chat request")?;
        rx.await
            .context(format!("receive chat response with {}", text))
    }

    fn new(cfg: Config) -> Result<Self, anyhow::Error> {
        let (tx, mut rx) = mpsc::unbounded_channel::<(String, oneshot::Sender<String>)>();
        tokio::task::spawn_blocking(move || {
            let context = match LLamaContext::new(&cfg.model, LLamaContextParam::from(cfg.context))
            {
                Ok(v) => v,
                Err(e) => {
                    warn!("create llama context error: {}", e);
                    return;
                }
            };
            let mut chat = match Chat::new(context, cfg.chat) {
                Ok(v) => v,
                Err(e) => {
                    warn!("create chatbot error: {}", e);
                    return;
                }
            };

            while let Some((input, tx)) = rx.blocking_recv() {
                if let Ok(output) = chat.chat(&input) {
                    let _ = tx.send(output);
                }
            }
        });
        Ok(Self { tx })
    }
}
