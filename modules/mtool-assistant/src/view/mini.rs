use dioxus::prelude::*;
use mapp::{
    anyhow,
    futures::{StreamExt, TryFutureExt},
    tracing::warn,
};

use crate::{
    media::{PlayerEvent, TimedCue},
    view::component::MediaPlayerControlContext,
};

#[component]
pub fn MiniView() -> Element {
    let context = MediaPlayerControlContext::get().suspend()?;

    let mut timed_cue = use_signal(|| TimedCue::default());

    use_resource(move || {
        async move {
            let mut stream = context().player_event_stream().await?;
            while let Some(Ok(event)) = stream.next().await {
                match event {
                    PlayerEvent::TimedCuesChanged { cue, .. } => {
                        timed_cue.set(cue);
                    }
                    _ => {}
                }
            }
            Ok::<_, anyhow::Error>(())
        }
        .unwrap_or_else(|e| warn!("{e:?}"))
    });

    rsx! {
        div {
            class: "flex flex-col items-center h-vh my-2 overflow-hidden",
            p {
                class: "text-2xl truncate",
                writing_mode: "vertical-lr",
                {
                    timed_cue().data.to_string()
                }
            }
        }
    }
}
