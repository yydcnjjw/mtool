use std::error::Error;

use dioxus::{prelude::*, CapturedError};
use mapp::{
    anyhow::{self, Context},
    prelude::*,
    tokio,
    tracing::{info, warn},
};
use mtool_dioxus::{components::WindowView, desktop::window, prelude::DioxusContext};

use crate::{
    music::{MediaPlayer, Netease, Playlist},
    notify::{NotifyContext, NotifyMode},
};

#[component]
pub fn MainView() -> Element {
    let notify_ctx: Res<NotifyContext> = use_context();
    use_hook(move || {
        window().set_visible(true);
        window().set_transparent(false);
        window().set_decorations(true);
        #[cfg(target_os = "windows")]
        {
            use mtool_dioxus::desktop::winit::platform::windows::WindowExtWindows;
            window().set_skip_taskbar(false);
        }
    });

    let notify_mode = notify_ctx.new_notify_mode_signal();

    let mut disabled = use_signal(|| false);

    let onclick = use_callback({
        move |_: Event<MouseData>| {
            to_owned![notify_ctx];
            spawn(async move {
                disabled.set(true);
                if let Err(e) =
                    NotifyContext::toggle_notify_mode_with_sync(notify_ctx.clone()).await
                {
                    warn!("{:?}", e);
                };
                disabled.set(false);
            });
        }
    });

    rsx! {
        WindowView {
            div {
                class: "flex flex-col items-center h-screen",
                button {
                    class: "btn btn-wide btn-xl",
                    disabled,
                    onclick,
                    {
                        match notify_mode() {
                            NotifyMode::Desktop => "Desktop",
                            NotifyMode::RemoteDesktop => "Remote Dekstop",
                        }
                    }
                }
                SuspenseBoundary {
                    fallback: |_| rsx! {
                        div { "Loading media player control" }
                    },
                    MediaPlayerControl {  }
                }

            }
        }
    }
}

#[component]
fn MediaPlayerControl() -> Element {
    let dioxus_context: Res<DioxusContext> = use_context();

    let player = use_resource(move || {
        to_owned![dioxus_context];
        async move { MediaPlayer::new(dioxus_context).await.unwrap() }
    })
    .suspend()?;

    {
        to_owned![player];
        let result = use_resource(move || {
            to_owned![player];
            async move {
                match tokio::task::spawn_blocking(move || {
                    let netease = Netease::new();
                    Ok::<_, anyhow::Error>(vec![
                        netease.get_playlist("71385702".into())?,
                        netease.get_playlist("60131".into())?,
                        netease.get_playlist("3001835560".into())?,
                    ])
                })
                .await
                .unwrap()
                {
                    Ok(playlists) => {
                        if let Err(e) = playlists
                            .into_iter()
                            .try_for_each(|playlist| player().add_playlist(playlist))
                        {
                            warn!("{e:?}");
                        }
                    }
                    Err(e) => {
                        warn!("{e:?}");
                    }
                }
            }
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
                        info!("play");
                        player().play().unwrap()
                    }
                },
                "play"
            },
            button {
                class: "btn",
                onclick: {
                    to_owned![player];
                    move |_| {
                        info!("pause");
                        player().pause().unwrap()
                    }
                },
                "pause"
            },
            input {
                class: "range",
                r#type: "range",
                min: "0",
                max: "100",
                value: format!("{}", player().volume().unwrap()),
                oninput: {
                    to_owned![player];
                    move|ev| {
                        if let Ok(value) = ev.value().parse::<usize>() {
                            player().set_volume(value).unwrap()
                        }
                    }
                }
            }
        }
    }
}
