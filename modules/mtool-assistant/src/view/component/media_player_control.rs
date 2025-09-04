use std::{any::type_name, collections::HashMap, pin::Pin, sync::Arc, time::Duration};

use dioxus::{
    core::{provide_root_context, SpawnIfAsync},
    prelude::*,
};
use mapp::{
    anyhow::{self, anyhow},
    futures::{Stream, StreamExt, TryFutureExt, TryStreamExt},
    prelude::*,
    tokio,
    tracing::warn,
};
use mtool_dioxus::{
    components::toast_err,
    free_icons::{
        icons::fa_solid_icons::{FaCloud, FaComputer, FaMobile, FaPause, FaPlay, FaVolumeHigh},
        Icon,
    },
    prelude::*,
    primitives::{switch::Switch, toast::use_toast},
};
use mtool_p2p::{self as p2p, gossipsub::IdentTopic};
use mtool_storage::lww;

use crate::{
    media::{MediaMetadata, MediaPlayer, Player, PlayerEvent, PlayerService, TimedCue},
    model::{ChatPrompt, ChatQuery, NeteaseViewModel},
    view::component::AiChatPreview,
};

#[derive(Clone)]
pub struct MediaPlayerControlContext {
    pub id: String,
    pub online_player_id: lww::State<String>,
    pub media_metadata: lww::State<MediaMetadata>,

    pub current_timed_cue: lww::State<(String, TimedCue)>,

    pub player: Signal<Option<Arc<MediaPlayer>>>,
    pub volume: Signal<f64>,

    pub subject: Arc<p2p::Subject<PlayerEvent>>,
}

impl MediaPlayerControlContext {
    pub fn get() -> Resource<Self> {
        use_resource(move || async move {
            async move {
                let this = match try_consume_context::<Self>() {
                    Some(this) => this,
                    None => provide_root_context(Self {
                        id: rand_string(),
                        online_player_id: lww::State::new(
                            consume_app_context().await,
                            "assistant.online_player_id",
                        )
                        .await?,
                        media_metadata: lww::State::new(
                            consume_app_context().await,
                            "assistant.media_metadata",
                        )
                        .await?,
                        current_timed_cue: lww::State::new(
                            consume_app_context().await,
                            "assistant.current_timed_cue",
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

    pub async fn player_event_stream(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<PlayerEvent, anyhow::Error>> + Send>>, anyhow::Error>
    {
        Ok(if let Some(player) = (self.player)() {
            player.listen().await?.map_err(|e| anyhow!("{e:?}")).boxed()
        } else {
            self.subject
                .stream()
                .map(|msg| msg.map(|msg| msg.data))
                .boxed()
        })
    }
}

#[component]
pub fn MediaPlayerControl() -> Element {
    let toast = use_toast();

    let context = MediaPlayerControlContext::get().suspend()?;

    let online_player_id = use_lww_signal(context().online_player_id.clone());

    let media_metadata = use_lww_signal(context().media_metadata.clone());

    // let mut cues = use_signal(|| Vec::new());
    let timed_cue = use_lww_signal(context().current_timed_cue);

    // use_effect(move || {
    //     _ = media_metadata.read(); // watch changed
    //     cues.clear();
    // });

    // use_effect(move || {
    //     cues.insert(timed_cue());
    // });

    let is_native = use_memo(move || online_player_id() == context().id);

    use_effect(move || {
        if is_native() {
            try_load_player_and_media(context())
                .unwrap_or_else(|e| warn!("{e:?}"))
                .spawn()
        }
    });

    use_resource(move || {
        let mut context = context();
        let player = (context.player)();
        async move {
            if is_native() {
                if let Some(player) = player {
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
                }
            }
            Ok::<_, anyhow::Error>(())
        }
        .unwrap_or_else(|e| warn!("{e:?}"))
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
            class: "flex flex-col items-center justify-stretch h-full",
            div {
                class: "flex flex-row items-center justify-stretch w-full gap-2 shrink-0",
                img {
                    class: "mask mask-circle shrink-0 w-36 h-36",
                    src: pic_url,
                }
                div {
                    class: "prose dark:prose-invert basis-full",
                    h4 {
                        class: "text-primary-content text-pretty",
                        { title.clone() }
                    }
                    p {
                        class: "text-sm text-base-content/50 text-pretty",
                        { album }
                    }
                    p {
                        class: "text-sm text-base-content/50 text-pretty",
                        { artist.clone() }
                    }
                }
            }
            div {
                class: "divider"
            }
            if !title.is_empty() && !artist.is_empty() {
                div {
                    class: "w-full text-pretty basis-full overflow-y-auto",
                    AiChatPreview {
                        prompt: ChatPrompt::Query(ChatQuery {
                            content: "歌曲出处".into(),
                            conds: vec![
                                ("歌曲名".into(), title),
                                ("作者/出品方".into(), artist),
                            ],
                        })
                    }
                }
            }
            // div {
            //     class: "flex flex-col gap-2 w-full text-center truncate basis-full overflow-y-auto",
            //     // for cue in cues() {
            //     //     p {
            //     //         { cue.data.to_string() }
            //     //     }
            //     // }
            // }
            div {
                class: "divider"
            }
            div {
                class: "flex flex-row w-full justify-center items-center shrink-0 mb-2",
                Switch {
                    class: "btn btn-circle btn-ghost swap aria-checked:swap-active",
                    checked: is_native(),
                    on_checked_change: move |is_native| {
                        if is_native {
                            context.online_player_id.set(context.id.clone());
                        }
                    },
                    if cfg!(feature = "desktop") {
                        Icon {
                            class: "swap-off fill-current",
                            width: 24,
                            height: 24,
                            icon: FaComputer,
                        }
                    } else {
                        Icon {
                            class: "swap-off fill-current",
                            width: 24,
                            height: 24,
                            icon: FaMobile,
                        }
                    }
                    Icon {
                        class: "swap-on fill-current",
                        width: 24,
                        height: 24,
                        icon: FaCloud,
                    }
                }
                Switch {
                    class: "btn btn-circle btn-ghost swap aria-checked:swap-active",
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
                    Icon {
                        class: "swap-off fill-current",
                        width: 24,
                        height: 24,
                        icon: FaPlay,
                    }
                    Icon {
                        class: "swap-on fill-current",
                        width: 24,
                        height: 24,
                        icon: FaPause,
                    }
                }
                div {
                    class: "flex flex-row items-center h-[32px]",
                    button {
                        class: "btn btn-circle btn-ghost",
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
            to_owned![context.subject, player];
            let mut online_player_id = context.online_player_id.subscribe();

            let mut stream = player.listen().await?;
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(Ok(ev)) = stream.next() => match ev {
                            PlayerEvent::MediaMetadataChanged { metadata } => {
                                context.media_metadata.set(metadata)
                            }
                            PlayerEvent::TimedCuesChanged { track_id, cue } => {
                                context.current_timed_cue.set((track_id, cue));
                            }
                            _ => {
                                subject
                                    .publish(&ev)
                                    .await
                                    .unwrap_or_else(|e| warn!("{e:?}"));
                            }
                        },
                        Ok(()) = online_player_id.changed() => {
                            if *online_player_id.borrow_and_update() != context.id {
                                player.pause().await.unwrap_or_else(|e| warn!("{e:?}"));
                            }
                        }
                        else => break,
                    }
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
