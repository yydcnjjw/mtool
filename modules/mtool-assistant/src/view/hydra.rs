use dioxus::{
    core::{has_context, use_hook_with_cleanup},
    prelude::*,
    router::RouterContext,
};
use mapp::{anyhow, futures::future};
use mtool_dioxus::{
    free_icons::{
        icons::fa_solid_icons::{FaMusic, FaNetworkWired},
        Icon,
    },
    generate_keymap, local_action,
    prelude::*,
};

use crate::view::component::{MediaPlayerControl, P2pDashboard};

#[derive(Clone)]
struct HydraContext {
    router: Signal<Option<RouterContext>>,
}

#[component]
pub fn HydraView() -> Element {
    use_context_provider(|| HydraContext {
        router: Signal::new(None),
    });

    init_keybinding()?;

    rsx! {
        Router::<Route> { }
    }
}

#[component]
fn HydraNav() -> Element {
    use_hook(|| {
        consume_context::<HydraContext>().router.set(try_router());
    });

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

fn init_keybinding() -> Result<(), RenderError> {
    let keybinding = use_context::<Keybinding>();

    let link_to = {
        let cb = use_callback(move |target| -> Result<(), anyhow::Error> {
            if let Some(HydraContext { router }) = has_context::<HydraContext>() {
                if let Some(router) = router() {
                    _ = router.replace(target);
                }
            }

            Ok(())
        });
        move |target: Route| move || future::ready(cb.call(target.clone()))
    };

    use_hook_with_cleanup(
        move || {
            let name = "assistant.hydra";
            let km = generate_keymap!(
                ("m", local_action!(link_to(Route::MediaPlayerControl {}))),
                ("p", local_action!(link_to(Route::P2pDashboard {}))),
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
