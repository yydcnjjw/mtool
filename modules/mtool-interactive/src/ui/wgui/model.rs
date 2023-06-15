use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionItem {
    pub id: usize,
    pub view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionExit {
    Id(usize),
    Completed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputContent {
    Plain(String),
    None,
}
