use dioxus::{
    core::{provide_root_context, use_hook_with_cleanup, SpawnIfAsync},
    prelude::*,
};
use mapp::{
    anyhow,
    futures::{StreamExt, TryFutureExt},
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
    generate_keymap, local_action,
    prelude::*,
    primitives::{switch::Switch, toast::use_toast},
};
use mtool_storage::lww;
use std::{any::type_name, sync::Arc};

use crate::{
    media::{MediaMetadata, MediaPlayer, PlaybackState, Player, PlayerEvent, TimedCue},
    model::{ChatPrompt, ChatQuery, NeteaseViewModel},
    view::component::AiChatPreview,
};

#[derive(Clone)]
pub struct MediaPlayerControlContext {
    pub id: String,
    pub online_player_id: lww::State<Option<String>>,
    pub media_metadata: lww::State<MediaMetadata>,
    pub current_timed_cue: lww::State<(String, TimedCue)>,
    pub playback_state: lww::State<PlaybackState>,

    pub player: Signal<Option<Arc<MediaPlayer>>>,
    pub volume: Signal<f64>,
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
                            "assistant.player.online_player_id",
                        )
                        .await?,
                        media_metadata: lww::State::new(
                            consume_app_context().await,
                            "assistant.player.media_metadata",
                        )
                        .await?,
                        current_timed_cue: lww::State::new(
                            consume_app_context().await,
                            "assistant.player.current_timed_cue",
                        )
                        .await?,
                        playback_state: lww::State::new(
                            consume_app_context().await,
                            "assistant.player.playback_state",
                        )
                        .await?,
                        player: Signal::new_in_scope(None, ScopeId::ROOT),
                        volume: Signal::new_in_scope(0., ScopeId::ROOT),
                    }),
                };

                Ok::<_, anyhow::Error>(this)
            }
            .await
            .expect(&format!("{}", type_name::<Self>()))
        })
    }

    fn is_online(&self) -> bool {
        self.online_player_id.borrow().as_ref() == Some(&self.id)
    }

    fn set_online(&self, online: bool) {
        if online {
            self.online_player_id.set(Some(self.id.clone()));
        } else {
            self.online_player_id.set(None);
        }
    }

    async fn set_play_pause(&self, play: bool) -> Result<(), anyhow::Error> {
        if let Some(player) = (self.player)() {
            if play {
                player.play().await
            } else {
                player.pause().await
            }
        } else {
            Ok(())
        }
    }
}

#[component]
pub fn MediaPlayerControl() -> Element {
    let toast = use_toast();

    let context = MediaPlayerControlContext::get().suspend()?;

    let netease_view_model = use_resource(move || NeteaseViewModel::new()).suspend()?;

    init_keybinding(context.into())?;

    let online_player_id = use_lww_signal(context().online_player_id.clone());

    let media_metadata = use_lww_signal(context().media_metadata.clone());

    let playback_state = use_lww_signal(context().playback_state.clone());

    let is_online = use_memo(move || online_player_id() == Some(context().id));
    let is_playing = use_memo(move || matches!(playback_state(), PlaybackState::Playing));

    use_effect(move || {
        if is_online() {
            try_load_player_and_media(context(), netease_view_model())
                .unwrap_or_else(|e| warn!("{e:?}"))
                .spawn()
        }
    });

    use_resource(move || {
        let mut context = context();
        let player = (context.player)();
        async move {
            if is_online() {
                if let Some(player) = player {
                    let metadata = player
                        .current_media_item()
                        .await?
                        .map(|item| item.metadata)
                        .flatten()
                        .unwrap_or_default();
                    context.media_metadata.set(metadata);

                    context.playback_state.set(player.playback_state().await?);

                    player.set_volume(0.1).await?;
                    // TODO: volume changed event
                    context.volume.set(0.1);
                }
            }
            Ok::<_, anyhow::Error>(())
        }
        .unwrap_or_else(|e| warn!("{e:?}"))
    });

    let MediaMetadata {
        title,
        artist,
        album,
        pic_url,
        ..
    } = media_metadata();

    let volume = ((context.read().volume)().clamp(0., 1.) * 100.).round() as usize;

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
            div {
                class: "divider"
            }
            div {
                class: "flex flex-row w-full justify-center items-center shrink-0 mb-2",
                button {
                    class: "btn btn-ghost",
                    onclick: |_| {
                        document::eval("window.open('https://music.163.com', '_blank', 'popup=true')");
                    },
                    "login"
                }
                Switch {
                    class: "btn btn-circle btn-ghost swap aria-checked:swap-active",
                    checked: is_online(),
                    on_checked_change: move |is_online| {
                        context.read().set_online(is_online);
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
                        context.read().set_play_pause(value).unwrap_or_else(|e| toast_err(toast, e)).await;
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

fn init_keybinding(context: ReadSignal<MediaPlayerControlContext>) -> Result<(), RenderError> {
    let keybinding = use_context::<Keybinding>();
    let toast = use_toast();

    let toggle_online_player = use_callback(move |_| {
        let ctx = context.read();
        ctx.set_online(!ctx.is_online());
        Ok(())
    });

    let toggle_play_pause = use_callback(move |_| {
        spawn(async move {
            let ctx = context.read();
            ctx.set_play_pause(!matches!(
                *ctx.playback_state.borrow(),
                PlaybackState::Playing
            ))
            .unwrap_or_else(|e| toast_err(toast, e))
            .await;
        });
        Ok(())
    });

    use_hook_with_cleanup(
        move || {
            let name = "assistant.media_player_control";
            let km = generate_keymap!(
                ("n", local_action!(toggle_online_player)),
                ("p", local_action!(toggle_play_pause)),
            )
            .unwrap();

            keybinding.push_keymap(name, km);
            (name, keybinding)
        },
        move |(name, keybinding)| {
            keybinding.remove_keymap(&name);
        },
    );
    Ok(())
}

async fn try_load_player_and_media(
    mut context: MediaPlayerControlContext,
    netease: NeteaseViewModel,
) -> Result<(), anyhow::Error> {
    let dioxus_context = consume_context::<DioxusContext>();

    if (context.player)().is_none() {
        let player = create_player(dioxus_context.clone()).await?;

        let items = netease
            .load_media_items_from_playlist(&[// 71385702, 60131, 3001835560
            ])
            .await?;
        player.set_media_items(items).await?;

        {
            to_owned![player];
            let mut online_player_id = context.online_player_id.subscribe();

            let mut stream = player.listen().await?;
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(Ok(ev)) = stream.next() => match ev {
                            PlayerEvent::MediaMetadataChanged{ metadata } => { context.media_metadata.set(metadata) }
                            PlayerEvent::TimedCuesChanged{ track_id, cue}=>{ context.current_timed_cue.set((track_id,cue)) }
                            PlayerEvent::PlaybackStateChanged { state } => { context.playback_state.set(state) },
                        },
                        Ok(()) = online_player_id.changed() => {
                            if online_player_id.borrow_and_update().as_ref() != Some(&context.id) {
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
