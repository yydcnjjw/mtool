use std::time::Duration;

use dioxus::prelude::*;
use mapp::{anyhow, futures::TryFutureExt, prelude::*, tokio, tracing::warn};
use mtool_dioxus::{
    free_icons::{icons::fa_solid_icons::FaNetworkWired, Icon},
    hooks::{consume_app_context, use_lww_signal},
};
use mtool_p2p as p2p;

use crate::view::component::MediaPlayerControlContext;

#[component]
pub fn MiniView() -> Element {
    let context = MediaPlayerControlContext::get().suspend()?;

    let timed_cue = use_lww_signal(context().current_timed_cue).map(|(_, cue)| cue);
    let stats = use_p2p_stats();

    let n_peers = stats
        .as_ref()
        .map(|stats| stats.gossipsub.all_peers.len())
        .unwrap_or_default();

    rsx! {
        div {
            class: "h-screen w-screen flex flex-col items-center my-2 overflow-hidden",
            div {
                class: "indicator",
                span {
                    class: "indicator-item indicator-middle size-[16px] text-xs",
                    "{n_peers}",
                }
                button {
                    class: "btn btn-square btn-ghost",
                    Icon {
                        width: 16,
                        height: 16,
                        icon: FaNetworkWired,
                    }
                }
            }
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
