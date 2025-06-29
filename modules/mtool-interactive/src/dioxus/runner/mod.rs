use dioxus::prelude::*;
use mapp::{
    anyhow,
    dpi::{PhysicalPosition, PhysicalSize},
};
use mtool_dioxus::{desktop::window, generate_keymap, local_action, prelude::*};

async fn hide_window() -> Result<(), anyhow::Error> {
    window().set_visible(false);
    Ok(())
}

pub fn view(_: &RouteParams) -> Element {
    use_hook(init_window);

    rsx! {
        div {
            class: "flex flex-col h-screen",
            onmousedown: move |e: Event<MouseData> | {
                if e.modifiers().shift() {
                    let _ = window().drag_window();
                }
            },
            div {
                class: "flex items-center",
                label {
                    class: "input input-ghost focus-within:outline-none stroke-primary w-full h-[64]",
                    svg {
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        class: "h-[1.5rem] opacity-50",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
                        }
                    }

                    input {
                        onmounted: |e| async move {
                            let _ = e.data().set_focus(true).await;
                        },
                        id: "",
                        class: "text-base",
                        r#type: "text",
                        placeholder: "Input: ",
                        tabindex: -1
                    }
                }
            },
            div {
                class: "divider m-0 h-0 mb-[8]"
            }
            div {
                tabindex: -1,
                class: "overflow-auto",
                div {
                    class: "flex flex-col ml-[12] mr-[12]",
                    for i in (0..5) {
                        div {
                            class: "h-[56]",
                            ResultItem {
                                i: i
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ResultList() -> Element {
    rsx! {}
}

#[component]
fn ResultItem(i: usize) -> Element {
    rsx! {
        div {
            class: "w-full h-full flex flex-row items-center focus:bg-base-content/10 rounded-sm outline-none",
            tabindex: -1,
            div {
                class: "size-[28] m-[16] bg-red-500",
            },
            div {
                class: "flex flex-col",
                span {
                    class: "text-lg",
                    "Emacs clients {i}",
                },
                span {
                    class: "text-sm text-[#909090]",
                    "Emacs clients {i}",
                }
            }
        }
    }
}
