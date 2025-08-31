use std::{any::type_name, rc::Rc, sync::Arc, time::Duration};

use dioxus::{
    core::{provide_root_context, SpawnIfAsync},
    prelude::*,
};
use mapp::{
    anyhow::{self, anyhow},
    futures::{future, StreamExt, TryFutureExt, TryStreamExt},
    prelude::*,
    tokio,
    tracing::{info, warn},
};
use mtool_dioxus::{
    free_icons::{
        icons::fa_solid_icons::{FaCloud, FaComputer, FaMobile, FaPause, FaPlay, FaVolumeHigh},
        Icon, IconShape,
    },
    prelude::*,
    primitives::{
        switch::Switch,
        toast::{use_toast, ToastOptions, Toasts},
    },
};
use mtool_p2p::{self as p2p, gossipsub::IdentTopic};
use mtool_storage::crdt;

use crate::{
    media::{MediaMetadata, MediaPlayer, Player, PlayerEvent, PlayerService},
    model::NeteaseViewModel,
};

#[derive(Clone)]
struct MediaPlayerControlContext {
    id: String,
    online_player_id: crdt::State<String>,
    media_metadata: crdt::State<MediaMetadata>,

    player: Signal<Option<Arc<MediaPlayer>>>,
    volume: Signal<f64>,

    subject: Arc<p2p::Subject<PlayerEvent>>,
}

impl MediaPlayerControlContext {
    fn get() -> Resource<Self> {
        use_resource(move || async move {
            async move {
                let this = match try_consume_context::<Self>() {
                    Some(this) => this,
                    None => provide_root_context(Self {
                        id: rand_string(),
                        online_player_id: crdt::State::new(
                            consume_app_context().await,
                            "assistant.online_player_id",
                        )
                        .await?,
                        media_metadata: crdt::State::new(
                            consume_app_context().await,
                            "assistant.media_metadata",
                        )
                        .await?,
                        player: Signal::new_in_scope(None, ScopeId::ROOT),
                        volume: Signal::new_in_scope(0., ScopeId::ROOT),
                        subject: Arc::new(
                            consume_app_context::<Res<p2p::Peer>>()
                                .await
                                .subscribe(&IdentTopic::new("PLAYER_EVENT"))
                                .await?,
                        ),
                    }),
                };

                Ok::<_, anyhow::Error>(this)
            }
            .await
            .expect(&format!("{}", type_name::<Self>()))
        })
    }
}

#[component]
pub fn MediaPlayerControl() -> Element {
    let toast = use_toast();

    let context = MediaPlayerControlContext::get().suspend()?;

    let online_player_id = use_crdt_signal(context().online_player_id.clone());
    let media_metadata = use_crdt_signal(context().media_metadata.clone());

    let is_native = use_memo(move || online_player_id() == context().id);

    use_effect(move || {
        if is_native() {
            try_load_player_and_media(context())
                .unwrap_or_else(|e| warn!("{e:?}"))
                .spawn()
        } else {
            if let Some(player) = (context().player)() {
                spawn(
                    async move { player.pause().await }
                        .unwrap_or_else(move |e| toast_err(toast, e)),
                );
            }
        }
    });

    use_effect(move || {
        let mut context = context();
        let player = (context.player)();
        async move {
            let mut stream = if let Some(player) = player {
                let metadata = player
                    .current_media_item()
                    .await?
                    .map(|item| item.metadata)
                    .flatten()
                    .unwrap_or_default();
                context.media_metadata.set(metadata);

                player.set_volume(0.1).await?;
                // TODO: volume changed event
                context.volume.set(0.1);

                player.listen().await?.map_err(|e| anyhow!("{e:?}")).boxed()
            } else {
                context
                    .subject
                    .stream()
                    .map(|msg| msg.map(|msg| msg.data))
                    .boxed()
            };

            while let Some(Ok(event)) = stream.next().await {
                match event {
                    PlayerEvent::MediaMetadataChanged { metadata } => {
                        context.media_metadata.set(metadata)
                    }
                    PlayerEvent::TimedCuesChanged { track_id, cue } => {
                        info!("{track_id}: {cue:?}");
                    }
                }
            }

            Ok::<_, anyhow::Error>(())
        }
        .unwrap_or_else(move |e| warn!("{e:?}"))
        .spawn()
    });

    let mut is_playing = use_signal(|| false);

    let context = context();

    let MediaMetadata {
        title,
        artist,
        album,
        pic_url,
        ..
    } = media_metadata();

    let volume = ((context.volume)().clamp(0., 1.) * 100.).round() as usize;

    rsx! {
        div {
            class: "flex flex-col items-center",

            div {
                class: "flex flex-row",
                div {
                    class: "stack w-[128px] h-[128px]",
                    img {
                        class: "mask mask-circle",
                        src: pic_url,
                    }
                }
                div {
                    class: "prose dark:prose-invert",
                    h4 {
                        class: "text-primary-content",
                        { title }
                    }
                    p {
                        class: "text-sm text-base-content/50",
                        { album }
                    }
                    p {
                        class: "text-sm text-base-content/50",
                        { artist }
                    }
                }
            }
            div {
                class: "flex flex-row items-center",
                Switch {
                    class: "swap aria-checked:swap-active",
                    checked: is_native(),
                    on_checked_change: move |is_native| {
                        if is_native {
                            context.online_player_id.set(context.id.clone());
                        }
                    },
                    button {
                        class: "btn btn-circle swap-off fill-current",
                        if cfg!(feature = "desktop") {
                            Icon {
                                width: 24,
                                height: 24,
                                icon: FaComputer,
                            }
                        } else {
                            Icon {
                                width: 24,
                                height: 24,
                                icon: FaMobile,
                            }
                        }
                    }
                    button {
                        class: "btn btn-circle swap-on fill-current",
                        Icon {
                            width: 24,
                            height: 24,
                            icon: FaCloud,
                        }
                    }
                }
                Switch {
                    class: "swap aria-checked:swap-active",
                    checked: is_playing(),
                    on_checked_change: move |value| async move {
                        is_playing.set(value);
                        if let Some(player) = (context.player)() {
                            if value {
                                player.play()
                            } else {
                                player.pause()
                            }.unwrap_or_else(|e| toast_err(toast, e))
                                .await
                        }
                    },
                    button {
                        class: "btn btn-circle swap-off fill-current",
                        Icon {
                            width: 24,
                            height: 24,
                            icon: FaPlay,
                        }
                    }
                    button {
                        class: "btn btn-circle swap-on fill-current",
                        Icon {
                            width: 24,
                            height: 24,
                            icon: FaPause,
                        }
                    }
                }
                div {
                    class: "flex flex-row items-center h-[32px]",
                    button {
                        class: "btn btn-circle",
                        Icon {
                            width: 24,
                            height: 24,
                            icon: FaVolumeHigh,
                        }
                    }
                    div {
                        class: "text-lg text-center",
                        "{ volume }%"
                    }
                }
            }
        }
    }
}

async fn try_load_player_and_media(
    mut context: MediaPlayerControlContext,
) -> Result<(), anyhow::Error> {
    let player_service = consume_app_context::<Res<PlayerService>>().await;
    let dioxus_context = use_context::<DioxusContext>();
    let netease = use_context_provider(|| NeteaseViewModel::new());

    if (context.player)().is_none() {
        let player = create_player(dioxus_context.clone()).await?;
        player_service.set_player(player.clone());

        let items = netease
            .load_media_items_from_playlist(&[71385702, 60131, 3001835560])
            .await?;
        player.set_media_items(items).await?;

        {
            to_owned![context.subject];
            let mut stream = player.listen().await?;
            tokio::spawn(async move {
                while let Some(Ok(ev)) = stream.next().await {
                    subject
                        .publish(&ev)
                        .await
                        .unwrap_or_else(|e| warn!("{e:?}"));
                }
            });
        }

        context.player.set(Some(player.clone()));
    }

    Ok(())
}

async fn create_player(dioxus_context: DioxusContext) -> Result<Arc<MediaPlayer>, anyhow::Error> {
    Ok(Arc::new(MediaPlayer::new(dioxus_context).await?))
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
