use dioxus::prelude::*;
use dioxus_desktop::window;
use mapp::{anyhow, prelude::*, tracing::warn};

use crate::notify::{NotifyContext, NotifyMode};

#[component]
pub fn MainView() -> Element {
    let notify_ctx: Res<NotifyContext> = use_context();

    use_hook(move || {
        window().set_visible(true);
        window().set_transparent(false);
        window().set_decorations(true);
        #[cfg(target_os = "windows")]
        {
            use dioxus_desktop::winit::platform::windows::WindowExtWindows;
            window().set_skip_taskbar(false);
        }
    });

    let mut notify_mode = use_signal(|| notify_ctx.notify_mode());

    let mut disabled = use_signal(|| false);

    let onclick = use_callback({
        to_owned![notify_ctx];
        move |e: Event<MouseData>| {
            to_owned![notify_ctx];
            spawn(async move {
                disabled.set(true);
                let mode =
                    match NotifyContext::toggle_notify_mode_with_sync(notify_ctx.clone()).await {
                        Ok(mode) => {
                            notify_mode.set(mode);
                        }
                        Err(e) => {
                            warn!("{:?}", e);
                        }
                    };
                disabled.set(false);
            });
        }
    });

    rsx! {
        div {
            class: "flex flex-col items-center h-screen",
            button {
                class: "btn btn-wide btn-xl",
                disabled,
                onclick,
                { notify_mode_text(notify_mode()) }
            }
        }
    }
}

fn notify_mode_text(mode: NotifyMode) -> &'static str {
    match mode {
        NotifyMode::Desktop => "Desktop",
        NotifyMode::RemoteDesktop => "RemoteDesktop",
    }
}

fn from_text(mode: &str) -> Option<NotifyMode> {
    Some(match mode {
        "Desktop" => NotifyMode::Desktop,
        "RemoteDesktop" => NotifyMode::RemoteDesktop,
        _ => return None,
    })
}
