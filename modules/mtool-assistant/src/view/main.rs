use dioxus::prelude::*;
use mtool_dioxus::components::WindowView;

use crate::view::component::MediaPlayerControl;

#[component]
pub fn MainView() -> Element {
    rsx! {
        if cfg!(target_os = "android") {
            document::Meta {
                http_equiv: "Content-Security-Policy",
                content: "upgrade-insecure-requests"
            }
        }
        document::Stylesheet {
            href: asset!("/assets/style.css")
        },
        WindowView {
            DockView {}
        }
    }
}

#[component]
fn DockView() -> Element {
    let dock_mode = use_dock_mode();

    #[component]
    fn Hide() -> Element {
        rsx! {
            div { }
        }
    }

    #[component]
    fn Show() -> Element {
        rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! {
                    div { "Loading media player control" }
                },
                MediaPlayerControl {  }
            }
        }
    }

    rsx! {
        div {
            class: "@container h-screen flex flex-row-reverse",
            div {
                class: "w-screen @3xs:w-full transition-[width] duration-300 ease-in-out bg-transparent flex flex-col",

                match dock_mode() {
                    DockMode::Show => rsx! { Show {} },
                    DockMode::Hide => rsx! { Hide {} },
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum DockMode {
    Show,
    Hide,
}

fn use_dock_mode() -> Signal<DockMode> {
    #[allow(unused_mut)]
    let mut dock_mode = use_signal(|| {
        if cfg!(feature = "desktop") {
            DockMode::Hide
        } else {
            DockMode::Show
        }
    });

    #[cfg(feature = "desktop")]
    {
        use mapp::{
            dpi::{PhysicalPosition, PhysicalSize},
            tracing::warn,
        };
        use mtool_dioxus::desktop::{use_global_shortcut, window, HotKeyState};

        use_effect(move || {
            let (width, height) = match dock_mode() {
                DockMode::Show => (512, 512),
                DockMode::Hide => (48, 256),
            };

            let win = window();
            let _ = win.request_inner_size(PhysicalSize::new(width, height));

            if let Some(monitor) = win.current_monitor() {
                let monitor_size = monitor.size();
                win.set_outer_position(PhysicalPosition::new(monitor_size.width - width, 128));
            }

            win.set_visible(true);
            // win.focus_window();
        });

        _ = use_global_shortcut("alt+z", move |state| {
            if state == HotKeyState::Pressed {
                match dock_mode() {
                    DockMode::Show => {
                        dock_mode.set(DockMode::Hide);
                    }
                    DockMode::Hide => {
                        dock_mode.set(DockMode::Show);
                    }
                }
            }
        })
        .inspect_err(|e| warn!("{:?}", e));
    }

    dock_mode
}
