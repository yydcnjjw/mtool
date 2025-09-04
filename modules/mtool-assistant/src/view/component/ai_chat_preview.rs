use std::hash::{DefaultHasher, Hash, Hasher};

use dioxus::prelude::*;
use mapp::{
    anyhow,
    futures::TryFutureExt,
    prelude::*,
    rig::{message::Reasoning, streaming::StreamedAssistantContent},
    serde::{Deserialize, Serialize},
    serde_json,
    tokio_stream::StreamExt,
};
use mtool_dioxus::{
    components::toast_err, hooks::consume_app_context, primitives::toast::use_toast,
};
use mtool_storage::kv;

use crate::model::{Agent, ChatPrompt};

#[component]
pub fn AiChatPreview(prompt: ChatPrompt) -> Element {
    let toast = use_toast();

    let mut reasoning_content = use_signal(|| String::new());
    let mut is_reasoning = use_signal(|| false);

    let mut text_content = use_signal(|| String::new());

    use_future(move || {
        to_owned![prompt];
        async move {
            let kvstore = consume_app_context::<Res<kv::Store>>().await;
            let history = kvstore.bucket::<kv::Integer, String>(Some("assistant.ai_chat"))?;

            let key = calculate_hash(&prompt).into();

            if let Some(result) = history.get(&key)? {
                let ChatResult { text, reasoning } = serde_json::from_str::<ChatResult>(&result)?;
                reasoning_content.set(reasoning);
                text_content.set(text);
                return Ok(());
            }

            let mut stream = Agent::chat(prompt).await?;
            while let Some(content) = stream.next().await {
                match content? {
                    StreamedAssistantContent::Text(text) => {
                        is_reasoning.set(false);
                        text_content.write().push_str(&text.text);
                    }
                    StreamedAssistantContent::ToolCall(_) => {}
                    StreamedAssistantContent::Reasoning(Reasoning { reasoning, .. }) => {
                        is_reasoning.set(true);
                        reasoning_content.write().push_str(&reasoning.join(""));
                    }
                    StreamedAssistantContent::Final(_) => {
                        history.set(
                            &key,
                            &serde_json::to_string(&ChatResult {
                                reasoning: reasoning_content(),
                                text: text_content(),
                            })?,
                        )?;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        }
        .unwrap_or_else(move |e| toast_err(toast, e))
    });

    rsx! {
        div {
            details {
                class: "collapse collapse-arrow border-base-300 border",
                summary {
                    class: "collapse-title font-semibold",
                    "Thoughts",
                    if is_reasoning() {
                        span { class: "mx-4 loading loading-spinner loading-sm" }
                    }
                }
                article {
                    class: "collapse-content prose prose-sm dark:prose-invert",
                    dangerous_inner_html: markdown::to_html(&*reasoning_content.read())
                }
            }

            article {
                class: "prose prose-sm dark:prose-invert",
                dangerous_inner_html: markdown::to_html(&*text_content.read())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
struct ChatResult {
    text: String,
    reasoning: String,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
