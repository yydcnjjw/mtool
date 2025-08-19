use std::sync::Arc;

use dioxus::prelude::*;
use mapp::{
    anyhow,
    futures::TryFutureExt,
    itertools::Itertools,
    prelude::*,
    tokio,
    tracing::{info, warn},
};
use mtool_dioxus::prelude::*;

use crate::{
    context::{AssistantContext, AssistantMode},
    media::{MediaItem, MediaPlayer, Netease, Player, RemotePlayer},
};

#[component]
pub fn MediaPlayerControl() -> Element {
    let player = use_media_player()
        .suspend()?
        .map(|player| player.as_ref().unwrap());

    let volume = {
        to_owned![player];
        use_resource(move || {
            to_owned![player];
            async move { player().volume().await.unwrap() }
        })
        .suspend()?
    };

    {
        to_owned![player];
        use_resource(move || {
            to_owned![player];
            async move {
                let items = collect_media_items().await?;
                player().add_media_items(items).await
            }
            .unwrap_or_else(|_| ())
        })
        .suspend()?;
    }

    rsx! {
        div {
            class: "flex flex-row",
            button {
                class: "btn",
                onclick: {
                    to_owned![player];
                    move |_| {
                        to_owned![player];
                        async move {
                        info!("play");
                        player().play().await.unwrap()
                    }
                    }
                },
                "play"
            },
            button {
                class: "btn",
                onclick: {
                    to_owned![player];
                    move |_| {
                        to_owned![player];
                        async move {
                            info!("pause");
                            player().pause().await.unwrap()
                        }
                    }
                },
                "pause"
            },
            input {
                class: "range",
                r#type: "range",
                min: "0",
                max: "100",
                value: format!("{}", (volume().clamp(0., 1.) * 100.).round() as usize),
                oninput: {
                    to_owned![player];
                    move|ev| {
                        to_owned![player];
                        async move {
                            if let Ok(value) = ev.value().parse::<usize>() {
                                player().set_volume(value as f64 / 100.).await.unwrap()
                            }
                        }
                    }
                }
            }
        }
    }
}

type AnyPlayer = Arc<dyn Player + Send + Sync>;

fn use_media_player() -> Resource<Option<AnyPlayer>> {
    let dioxus_context: Res<DioxusContext> = use_context();
    let context: Res<AssistantContext> = use_context();

    let mode = context.mode_change_signal();

    use_resource(move || {
        to_owned![dioxus_context, context];
        async move {
            let use_remote = match mode() {
                AssistantMode::Desktop => !context.is_desktop(),
                AssistantMode::RemoteDesktop => context.is_desktop(),
            };

            let player: AnyPlayer = if use_remote && context.request_address().is_some() {
                Arc::new(RemotePlayer::connect(context.request_address().unwrap()).await?)
            } else {
                Arc::new(MediaPlayer::new(dioxus_context).await?)
            };

            Ok::<_, anyhow::Error>(Some(player))
        }
        .inspect_err(|e| warn!("{e:?}"))
        .unwrap_or_else(|_| None)
    })
}

async fn collect_media_items() -> Result<Vec<MediaItem>, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        let netease = Netease::new();
        let items = vec![
            netease.get_playlist("71385702".into())?,
            netease.get_playlist("60131".into())?,
            netease.get_playlist("3001835560".into())?,
        ]
        .into_iter()
        .map(|playlist| playlist.entries)
        .flatten()
        .map(|entry| MediaItem::new(entry.url))
        .collect_vec();

        Ok(items)
    })
    .await?
}
