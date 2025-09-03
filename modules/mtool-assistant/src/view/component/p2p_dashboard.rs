use std::time::Duration;

use dioxus::prelude::*;
use mapp::{anyhow, futures::TryFutureExt, prelude::*, tokio, tracing::warn};
use mtool_dioxus::{
    components::toast_err,
    hooks::{consume_app_context, use_app_context},
    primitives::toast::use_toast,
};
use mtool_p2p as p2p;

#[component]
pub fn P2pDashboard() -> Element {
    let toast = use_toast();
    let peer = use_app_context::<Res<p2p::Peer>>().suspend()?;
    let mut is_loading = use_signal(|| false);

    let stats = use_p2p_stats();

    if let Some(p2p::Stats {
        network_info,
        gossipsub,
        ..
    }) = stats()
    {
        rsx! {
            div {
                class: "flex flex-col",
                div {
                    class: "flex flex-row p-2",
                    button {
                        class: "btn btn-ghost",
                        disabled: is_loading(),
                        onclick: move |_| {
                            is_loading.set(true);
                            spawn(async move {
                                peer().bootstrap().unwrap_or_else(|e|toast_err(toast, e)).await;
                                is_loading.set(false);
                            });
                        },
                        "reload",
                        if is_loading() {
                            span {
                                class: "loading loading-spinner"
                            }
                        }
                    }
                }
                div {
                    class: "flex flex-row",
                    div {
                        class: "stats shadow",
                        div {
                            class: "stat",
                            div { class: "stat-title", "Connected peers" }
                            div { class: "stat-value", "{network_info.num_peers()}" }
                            div { class: "stat-desc", "The total number of connected peers" }
                        }
                    }
                }
                ul {
                    class: "list rounded-box shadow-md",
                    li {
                        class: "p-4 pb-2 text-xs opacity-60",
                        "Gossipsub all peers",
                        span {
                            class: "badge mx-2",
                            "{gossipsub.all_peers.len()}"
                        }
                    }

                    for (peer_id, topics) in gossipsub.all_peers.iter() {
                        li {
                            class: "list-row",
                            div {
                                div {
                                    class: "text-sm opacity-60 p-1",
                                    "{peer_id}"
                                }
                                div {
                                    class: "flex flex-row flex-wrap justify-stretch",
                                    for topic in topics {
                                        span {
                                            class: "badge badge-xs truncate max-w-32 mx-1",
                                            { topic.to_string() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn use_p2p_stats() -> ReadOnlySignal<Option<p2p::Stats>> {
    let mut stats = use_signal(|| None);
    use_future(move || {
        async move {
            let peer = consume_app_context::<Res<p2p::Peer>>().await;

            let mut tick = tokio::time::interval(Duration::from_secs(5));

            loop {
                tick.tick().await;
                stats.set(Some(peer.stats().await?));
            }
        }
        .unwrap_or_else(|e: anyhow::Error| warn!("{e:?}"))
    });
    stats.into()
}
