use dioxus::prelude::*;
use mapp::prelude::*;
use mtool_dioxus::{components::WindowView, desktop::window};

use crate::context::{AssistantContext, AssistantMode};

use super::MediaPlayerControl;

#[component]
pub fn MainView() -> Element {
    let ctx: Res<AssistantContext> = use_context();

    init_window();

    let mode = ctx.mode_change_signal();

    let onclick = use_callback({
        move |_: Event<MouseData>| {
            ctx.toggle_mode();
        }
    });

    rsx! {
        WindowView {
            div {
                class: "flex flex-col items-center h-screen",
                button {
                    class: "btn btn-wide btn-xl",
                    onclick,
                    {
                        match mode() {
                            AssistantMode::Desktop => "Desktop",
                            AssistantMode::RemoteDesktop => "Remote Dekstop",
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

fn init_window() {
    use_hook(move || {
        let win = window();
        win.set_visible(true);
        win.set_transparent(false);
        win.set_decorations(true);
        #[cfg(target_os = "windows")]
        {
            use mtool_dioxus::desktop::winit::platform::windows::WindowExtWindows;
            win.set_skip_taskbar(false);
        }
    });
}
