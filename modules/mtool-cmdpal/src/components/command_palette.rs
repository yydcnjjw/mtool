use std::rc::Rc;

use dioxus::{core::use_hook_with_cleanup, prelude::*};
use mapp::{
    anyhow,
    itertools::Itertools,
    prelude::*,
    tracing::{debug, warn},
};
use mtool_dioxus::{
    desktop::{use_global_shortcut, window, HotKeyState},
    free_icons::{icons::go_icons::GoSearch, Icon},
    generate_keymap, local_action,
    prelude::*,
};

use crate::{
    components::{CommandInput, CommandList},
    CommandPalette, CommandResult,
};

#[component]
pub fn CommandPaletteView() -> Element {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    init_window();

    let onmousedown = move |e: Event<MouseData>| {
        if e.modifiers().shift() {
            let _ = window().drag_window();
        }
    };

    let mut command_input = use_context_provider(|| Signal::new(CommandInput::new()));

    let oninput = move |text: String| {
        command_input.set(CommandInput::from(text));
    };

    let mut view = use_signal::<ViewFn>(|| default_view);

    let command_result = use_context_provider(|| Signal::new(CommandResult::DoNothing));

    init_keybinding(command_result);

    use_effect(move || {
        match &*command_result.read() {
            CommandResult::ShowView(v) => {
                view.set(*v);
            }
            CommandResult::ShowHome => {
                command_input.set(CommandInput::new());
                view.set(default_view);
            }
            CommandResult::Dismiss => {
                command_input.set(CommandInput::new());
                view.set(default_view);
                spawn(async move {
                    hide_window().await.unwrap();
                });
            }
            CommandResult::Hide => {
                spawn(async move {
                    hide_window().await.unwrap();
                });
            }
            CommandResult::DoNothing => {}
        };
    });

    rsx! {
        div {
            class: "flex flex-col h-screen",
            tabindex: -1,
            onmousedown,
            onmouseleave: |_| {
                let win = window();
                if !win.has_focus() {
                    window().set_visible(false);
                }
            },
            SearchBar {
                value: command_input().value,
                oninput,
            }
            div {
                class: "divider m-0 h-0 MB-[8]"
            }
            DynamicView { view }
        }
    }
}

fn default_view() -> Element {
    rsx! {
        DefaultView {  }
    }
}

#[component]
fn DefaultView() -> Element {
    let cmdpal: Res<CommandPalette> = use_context();

    let mut items = use_signal(Vec::new);

    use_hook(|| {
        spawn(async move {
            let mut rx = cmdpal.subscribe_commands_changed();
            items.set(
                rx.borrow()
                    .iter()
                    .map(|(_, items)| items.clone())
                    .flatten()
                    .collect_vec(),
            );

            while let Ok(_) = rx.changed().await {
                items.set(
                    rx.borrow()
                        .iter()
                        .inspect(|(source, items)| {
                            debug!("source: {}, count: {}", source, items.len());
                        })
                        .map(|(_, items)| items.clone())
                        .flatten()
                        .collect_vec(),
                );
            }
        });
    });

    rsx! {
        CommandList {
            items
        }
    }
}

type ViewFn = fn() -> Element;

#[component]
fn DynamicView(view: ReadSignal<ViewFn>) -> Element {
    view.read()()
}

#[component]
fn SearchBar(
    #[props(default)] value: String,
    #[props(default)] oninput: EventHandler<String>,
) -> Element {
    let keybinding = use_context::<Keybinding>();
    let mut input_node: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let search = use_callback(move |_| {
        if let Some(node) = input_node() {
            spawn(async move {
                _ = node.set_focus(true).await;
            });
        }
        Ok(())
    });

    use_hook_with_cleanup(
        || {
            let km = generate_keymap!(("C-s", local_action!(search)),).unwrap();
            keybinding.push_keymap("cmdpal.search_bar", km);
            keybinding
        },
        move |keybinding| {
            keybinding.remove_keymap("cmdpal");
        },
    );

    rsx! {
        div {
            class: "shrink-0 flex flex-row items-center h-[64] ml-[12] mr-[12]",
            Icon {
                class: "shrink-0 m-[16]",
                width: 20,
                height: 20,
                icon: GoSearch
            }
            input {
                onmounted: move |e| { input_node.set(Some(e.data())) },
                value,
                oninput: move |e| {
                    oninput.call(e.data().value())
                },
                class: "w-full text-base outline-none",
                r#type: "text",
                placeholder: "Type here to search ...",
                autofocus: true,
            }
        }
    }
}

async fn hide_window() -> Result<(), anyhow::Error> {
    window().set_visible(false);
    Ok(())
}

fn init_keybinding(mut command_result: Signal<CommandResult>) {
    let keybinding = use_context::<Keybinding>();

    if let Err(e) = use_global_shortcut("alt+Space", |state| {
        if state == HotKeyState::Pressed {
            let win = window();
            win.set_visible(true);
            win.focus_window();
        }
    }) {
        warn!("{:?}", e);
    }

    let dismiss = use_callback(move |_| {
        command_result.set(CommandResult::Dismiss);
        Ok(())
    });

    let show_home = use_callback(move |_| {
        command_result.set(CommandResult::ShowHome);
        Ok(())
    });

    use_hook_with_cleanup(
        || {
            let km = generate_keymap!(
                ("C-g", local_action!(dismiss)),
                ("<Escape>", local_action!(show_home)),
            )
            .unwrap();
            keybinding.push_keymap("cmdpal", km);
            keybinding
        },
        move |keybinding| {
            keybinding.remove_keymap("cmdpal");
        },
    );
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn init_window() {
    use mapp::dpi::{PhysicalPosition, PhysicalSize};
    use_hook(move || {
        let window = window();
        let window_size = PhysicalSize::new(800, 600);
        let _ = window.request_inner_size(window_size);

        if let Some(monitor) = window.current_monitor() {
            let monitor_size = monitor.size();
            let x = (monitor_size.width - window_size.width) / 2;
            let y = (monitor_size.height - window_size.height) / 2;
            window.set_outer_position(PhysicalPosition::new(x, y));
        }
    })
}
