use dioxus::prelude::*;

use crate::view::component::MediaPlayerControl;

#[component]
pub fn HydraView() -> Element {
    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {
                div { "Loading media player control" }
            },
            MediaPlayerControl {  }
        }
    }
}
