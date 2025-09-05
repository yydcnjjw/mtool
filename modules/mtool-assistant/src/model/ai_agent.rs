use dioxus::prelude::*;
use mapp::{
    anyhow,
    itertools::Itertools,
    prelude::*,
    reqwest,
    rig::{
        client::CompletionClient,
        message::Message,
        providers::gemini::{self, completion::gemini_api_types::*},
        streaming::{StreamingCompletion, StreamingCompletionResponse},
    },
    serde_json,
};
use mtool_core::ConfigStore;

#[derive(Clone)]
pub struct Agent {
    api_key: String,
}

impl Agent {
    async fn new(cs: Res<ConfigStore>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            api_key: cs.get("ai.gemini.api_key").await?,
        })
    }

    pub async fn client() -> Result<gemini::Client, anyhow::Error> {
        let agent = inject_once(&consume_context::<Injector>(), Agent::new).await??;
        Ok(gemini::Client::builder(&agent.api_key)
            .custom_client(reqwest::Client::builder().build()?)
            .build()?)
    }

    pub async fn chat(
        prompt: ChatPrompt,
    ) -> Result<
        StreamingCompletionResponse<gemini::streaming::StreamingCompletionResponse>,
        anyhow::Error,
    > {
        let client = Agent::client().await?;

        let chat = client
            .agent("gemini-2.5-flash-lite")
            .temperature(1.)
            .additional_params(serde_json::to_value(prompt.additional_parameters())?)
            .build();

        Ok(chat
            .stream_completion(prompt, Vec::new())
            .await?
            .stream()
            .await?)
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ChatPrompt {
    Query(ChatQuery),
}

impl ChatPrompt {
    fn additional_parameters(&self) -> AdditionalParameters {
        match self {
            ChatPrompt::Query(_) => AdditionalParameters::default().with_config(GenerationConfig {
                top_p: Some(0.95),
                candidate_count: Some(1),
                temperature: Some(0.),
                thinking_config: Some(ThinkingConfig {
                    include_thoughts: Some(true),
                    thinking_budget: 4096,
                }),
                ..Default::default()
            }),
        }
    }
}

impl Into<Message> for ChatPrompt {
    fn into(self) -> Message {
        match self {
            ChatPrompt::Query(ChatQuery { content, conds }) => format!(
                r#"查询: {content}, {}"#,
                conds.iter().map(|(k, v)| format!("{k}: {v}")).join(", "),
            )
            .trim()
            .into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ChatQuery {
    pub content: String,
    pub conds: Vec<(String, String)>,
}
