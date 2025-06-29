use std::fmt::Debug;

use mapp::{
    serde::{Deserialize, Serialize},
    serde_json,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "mapp::serde")]
pub struct CompletionItem {
    pub id: usize,
    pub template_id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum CompletionExit {
    Id(usize),
    Completed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum OutputContent {
    Plain(String),
    None,
}
