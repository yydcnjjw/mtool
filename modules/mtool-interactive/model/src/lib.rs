use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionMeta {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputContent {
    Plain(String),
    None
}
