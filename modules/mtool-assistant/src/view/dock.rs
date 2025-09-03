use dioxus::prelude::*;

use crate::view::{hydra::HydraView, mini::MiniView};

#[component]
pub fn DockView() -> Element {
    #[allow(unused_mut)]
    let mut dock_mode = use_signal(|| {
        if cfg!(feature = "desktop") {
            DockMode::Hide
        } else {
            DockMode::Show
        }
    });

    #[allow(unused_mut)]
    let mut window_top = use_signal(|| 256);

    #[cfg(feature = "desktop")]
    {
        use mapp::{
            dpi::{PhysicalPosition, PhysicalSize},
            tracing::warn,
        };
        use mtool_dioxus::desktop::{
            use_global_shortcut, use_wry_event_handler, window, winit::event::Event as WinitEvent,
            HotKeyState, WindowEvent,
        };

        use_effect(move || {
            let (width, height) = match dock_mode() {
                DockMode::Show => (512, 512),
                DockMode::Hide => (48, 512),
            };

            let win = window();
            let _ = win.request_inner_size(PhysicalSize::new(width, height));

            if let Some(monitor) = win.current_monitor() {
                let monitor_size = monitor.size();
                win.set_outer_position(PhysicalPosition::new(
                    monitor_size.width - width,
                    window_top(),
                ));
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

        use_wry_event_handler(move |ev, _| match ev {
            WinitEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(focused) => {
                    if !focused {
                        let old = window_top();
                        let new = window()
                            .outer_position()
                            .map(|pos| pos.y as u32)
                            .unwrap_or(old);
                        if old != new {
                            window_top.set(new);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        });
    }

    let onmousedown = move |e: Event<MouseData>| {
        use mtool_dioxus::desktop::window;
        if e.modifiers().shift() {
            let _ = window().drag_window();
        }
    };

    rsx! {
        div {
            class: "@container h-screen flex flex-row-reverse",
            onmousedown,
            div {
                class: "w-screen bg-transparent flex flex-col",
                // @3xs:w-full transition-[width] duration-300 ease-in-out

                SuspenseBoundary {
                    fallback: |_| rsx! {
                        div { "Loading ..." }
                    },
                    match dock_mode() {
                        DockMode::Show => rsx! { HydraView {} },
                        DockMode::Hide => rsx! { MiniView {} },
                    }
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
