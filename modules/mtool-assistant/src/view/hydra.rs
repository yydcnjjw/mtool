use dioxus::prelude::*;
use mtool_dioxus::free_icons::{
    icons::fa_solid_icons::{FaMusic, FaNetworkWired},
    Icon,
};

use crate::view::component::{MediaPlayerControl, P2pDashboard};

#[component]
pub fn HydraView() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn HydraNav() -> Element {
    rsx! {
        div {
            class: "h-screen w-screen flex flex-row py-2 overflow-hidden",
            ul {
                class: "menu-sm",
                li {
                    Link {
                        class: "tooltip tooltip-right",
                        "data-tip": "media player control",
                        to: Route::MediaPlayerControl {},
                        Icon {
                            class: "w-4 h-4",
                            icon: FaMusic,
                        }
                    }
                }
                li {
                    Link {
                        class: "tooltip tooltip-right",
                        "data-tip": "p2p dashboard",
                        to: Route::P2pDashboard {},
                        Icon {
                            class: "w-4 h-4",
                            icon: FaNetworkWired,
                        }
                    }
                }
            }
            SuspenseBoundary {
                fallback: |_| rsx! {
                    div {
                        class: "skeleton w-screen h-screen"
                    }
                },
                div {
                    class: "w-full h-full px-1 overflow-hidden",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(HydraNav)]
        #[route("/")]
        MediaPlayerControl {},
        #[route("/p2p")]
        P2pDashboard {},
}
