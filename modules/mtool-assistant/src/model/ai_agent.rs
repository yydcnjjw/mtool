use dioxus::prelude::*;
use mapp::{
    anyhow,
    itertools::Itertools,
    prelude::*,
    reqwest::{self, Proxy},
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
    model: String,
    https_proxy: Option<String>,
}

impl Agent {
    async fn new(cs: Res<ConfigStore>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            api_key: cs.get("ai.gemini.api_key")?,
            model: cs.get("ai.gemini.model")?,
            https_proxy: cs.get_optional("ai.https_proxy"),
        })
    }

    pub async fn client() -> Result<(gemini::Client, String), anyhow::Error> {
        let agent = inject_once(&consume_context::<Injector>(), Agent::new).await??;
        let mut client = reqwest::Client::builder();

        if let Some(https_proxy) = agent.https_proxy {
            client = client.proxy(Proxy::https(https_proxy)?);
        }

        Ok((
            gemini::Client::<reqwest::Client>::builder()
                .api_key(agent.api_key)
                .http_client(client.build()?)
                .build()?,
            agent.model.to_owned(),
        ))
    }

    pub async fn chat(
        prompt: ChatPrompt,
    ) -> Result<
        StreamingCompletionResponse<gemini::streaming::StreamingCompletionResponse>,
        anyhow::Error,
    > {
        let (client, model) = Agent::client().await?;

        let chat = client
            .agent(&model)
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
