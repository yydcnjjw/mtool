use std::hash::{DefaultHasher, Hash, Hasher};

use dioxus::prelude::*;
use mapp::{
    anyhow,
    futures::TryFutureExt,
    prelude::*,
    rig::streaming::StreamedAssistantContent,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio_stream::StreamExt,
    tracing::warn,
};
use mtool_dioxus::{
    components::toast_err,
    hooks::consume_app_context,
    primitives::toast::use_toast,
};
use mtool_storage::kv;

use crate::model::{Agent, ChatPrompt};

#[component]
pub fn AiChatPreview(prompt: ReadOnlySignal<ChatPrompt>) -> Element {
    let toast = use_toast();

    let history = use_resource(|| async move {
        let kvstore = consume_app_context::<Res<kv::Store>>().await;
        kvstore
            .bucket::<kv::Integer, String>(Some("assistant.ai_chat"))
            .expect("kvstore assistant.ai_chat")
    })
    .suspend()?;

    let mut is_chating = use_signal(|| false);

    let mut text_content = use_signal(|| String::new());

    let mut chat = use_effect(move || {
        let prompt = prompt();

        text_content.set("".into());

        spawn(
            async move {
                let history = history();
                let key = calculate_hash(&prompt).into();

                if let Some(result) = history.get(&key)? {
                    let ChatResult { text } = serde_json::from_str::<ChatResult>(&result)?;
                    text_content.set(text);
                    return Ok(());
                }

                is_chating.set(true);

                let mut stream = Agent::chat(prompt).await?;
                while let Some(content) = stream.next().await {
                    match content? {
                        StreamedAssistantContent::Text(text) => {
                            text_content.write().push_str(&text.text);
                        }
                        StreamedAssistantContent::Final(_) => {
                            is_chating.set(false);
                            history.set(
                                &key,
                                &serde_json::to_string(&ChatResult {
                                    text: text_content(),
                                })?,
                            )?;
                            _ = history.flush_async().await.inspect_err(|e| warn!("{e:?}"));
                        }
                        _ => {}
                    }
                }
                Ok::<(), anyhow::Error>(())
            }
            .unwrap_or_else(move |e| {
                toast_err(toast, e);
                is_chating.set(false);
            }),
        );
    });

    let update = use_callback(move |_| {
        let key = calculate_hash(&prompt()).into();
        _ = history().remove(&key);
        chat.mark_dirty();
    });

    rsx! {
        div {
            article {
                class: "prose prose-sm dark:prose-invert",
                h4 {
                    class: "flex justify-between items-center",
                    "歌曲出处",
                    button {
                        class: "btn btn-ghost btn-xs mx-4",
                        onclick: update,
                        if is_chating() {
                            span { class: "mx-4 loading loading-spinner loading-xs" }
                        }
                        "更新"
                    }
                }
                div {
                    dangerous_inner_html: markdown::to_html(&*text_content.read()),
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
struct ChatResult {
    text: String,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
