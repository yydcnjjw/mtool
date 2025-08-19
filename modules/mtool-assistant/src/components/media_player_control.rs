use std::{sync::Arc, time::Duration};

use dioxus::prelude::*;
use mapp::{
    anyhow,
    futures::TryFutureExt,
    itertools::Itertools,
    prelude::*,
    tokio,
    tracing::{info, warn},
};
use mtool_dioxus::{
    prelude::*,
    primitives::toast::{use_toast, ToastOptions, Toasts},
};

use crate::{
    context::{AssistantContext, AssistantMode},
    media::{AnyPlayer, MediaItem, MediaPlayer, Netease, PlayerService, RemotePlayer},
};

#[component]
pub fn MediaPlayerControl() -> Element {
    let player = create_player()?;
    let toast = use_toast();

    let volume = {
        use_resource(move || async move {
            player()
                .volume()
                .unwrap_or_else(move |e| {
                    toast_err(toast, e);
                    0.
                })
                .await
        })
        .suspend()?
    };

    let context = use_app_resource::<Res<AssistantContext>>().suspend()?;

    use_resource(move || {
        async move {
            if context().is_native() {
                let items = collect_media_items().await?;
                player().add_media_items(items).await?;
            }
            Ok::<_, anyhow::Error>(())
        }
        .unwrap_or_else(move |e| toast_err(toast, e))
    })
    .suspend()?;

    rsx! {
        div {
            class: "flex flex-row",
            button {
                class: "btn",
                onclick: {
                    move |_| async move { player().play().unwrap_or_else(|e| toast_err(toast, e)).await }
                },
                "play"
            },
            button {
                class: "btn",
                onclick: {
                    move |_| async move { player().pause().unwrap_or_else(|e| toast_err(toast, e)).await }
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
                    move|ev| {
                        async move {
                            if let Ok(value) = ev.value().parse::<usize>() {
                                player().set_volume(value as f64 / 100.).unwrap_or_else(|e| toast_err(toast, e)).await
                            }
                        }
                    }
                }
            }
        }
    }
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

pub fn create_player() -> Result<MappedSignal<AnyPlayer, Signal<Option<AnyPlayer>>>, RenderError> {
    let context = use_app_resource::<Res<AssistantContext>>().suspend()?;
    let player_service = use_app_resource::<Res<PlayerService>>().suspend()?;
    let dioxus_context = use_context::<DioxusContext>();

    let mode = context().mode_change_signal();
    use_resource(move || {
        to_owned![context, dioxus_context, player_service];
        async move {
            let player = get_player(mode(), context(), dioxus_context.clone())
                .await
                .unwrap();
            player_service().set_player(player.clone());
            player
        }
    })
    .suspend()
}

async fn try_get_remote_player(
    mode: AssistantMode,
    context: Res<AssistantContext>,
) -> Option<RemotePlayer> {
    let use_remote = match mode {
        AssistantMode::Desktop => !context.is_desktop(),
        AssistantMode::RemoteDesktop => context.is_desktop(),
    };

    if use_remote {
        let request_address = context.request_address()?;
        RemotePlayer::connect(request_address)
            .await
            .inspect(|_| info!("use remote player"))
            .inspect_err(|e| warn!("{e:?}"))
            .ok()
    } else {
        None
    }
}

async fn get_player(
    mode: AssistantMode,
    context: Res<AssistantContext>,
    dioxus_context: DioxusContext,
) -> Result<AnyPlayer, anyhow::Error> {
    Ok(match try_get_remote_player(mode, context).await {
        Some(player) => Arc::new(player) as AnyPlayer,
        None => Arc::new(
            MediaPlayer::new(dioxus_context)
                .await
                .inspect(|_| info!("use native player"))?,
        ),
    })
}

fn toast_err<E>(toast: Toasts, e: E)
where
    E: Into<anyhow::Error>,
{
    toast.error(
        "Error occurred".into(),
        ToastOptions::default()
            .description(e.into())
            .duration(Duration::from_secs(3)),
    );
}
