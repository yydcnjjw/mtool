use std::time::Duration;

use dioxus::prelude::*;
use mapp::{anyhow, futures::TryFutureExt, prelude::*, tokio, tracing::warn};
use mtool_dioxus::hooks::consume_app_context;
use mtool_p2p as p2p;

#[component]
pub fn P2pDashboard() -> Element {
    let stats = use_p2p_stats();

    let n_peers = stats
        .as_ref()
        .map(|stats| stats.gossipsub.all_peers.len())
        .unwrap_or_default();

    rsx! {
        div {
            class: "flex flex-col",
            div { class: "stats shadow",
                  div { class: "stat",
                        div { class: "stat-title", "Gossipsub all peers" }
                        div { class: "stat-value", "{n_peers}" }
                  }
            }
        }
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
