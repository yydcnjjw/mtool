use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::RequestMeta;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub role: String,

    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,

    pub messages: Vec<ChatMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub logit_bias: HashMap<String, f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".into(),
            messages: Default::default(),
            temperature: Default::default(),
            top_p: Default::default(),
            n: Default::default(),
            stream: Default::default(),
            stop: Default::default(),
            max_tokens: Default::default(),
            presence_penalty: Default::default(),
            frequency_penalty: Default::default(),
            logit_bias: Default::default(),
            user: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: usize,
    pub choices: Vec<ChatChoice>,
    pub usage: ChatUsage,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

impl RequestMeta for ChatRequest {
    fn path() -> &'static str {
        "v1/chat/completions"
    }

    fn api_name() -> &'static str {
        "chat_completion"
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }
}

#[cfg(test)]
mod tests {
    // use crate::openai::Client;

    use super::*;

    #[test]
    fn test_deserialize_chat_response() {
        let _: ChatResponse = serde_json::from_str(
            r#"{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "\n\nHello there, how may I assist you today?"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 9,
    "completion_tokens": 12,
    "total_tokens": 21
  }
}
"#,
        )
        .unwrap();
    }
}
