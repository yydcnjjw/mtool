use std::{ops::Deref, rc::Rc, sync::Arc};

use dioxus::{core::use_hook_with_cleanup, prelude::*};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use mapp::{anyhow, itertools::Itertools, rand::random, tracing::warn};
use mtool_dioxus::{generate_keymap, local_action, prelude::*};
use mtool_storage::kv;

use crate::{CommandItem, CommandResult};

#[derive(Clone)]
struct CommandListCtx {
    node_ref: Signal<Option<Rc<MountedData>>>,
    filtered_items: Signal<Vec<CommandItem>>,
    focus_item_index: Signal<usize>,

    command_result: Signal<CommandResult>,

    command_history: kv::Bucket<'static, String, String>,
}

impl CommandListCtx {
    fn is_selected(&self, idx: usize) -> bool {
        (self.focus_item_index)() == idx
    }

    async fn try_scroll_to_item(
        &self,
        item_id: String,
        item: Rc<MountedData>,
    ) -> Result<(), RenderError> {
        if let Some(parent) = (self.node_ref)() {
            let parent_rect = parent.get_client_rect().await?;
            let child_rect = item.get_client_rect().await?;

            if child_rect.min_y() >= parent_rect.max_y()
                || child_rect.max_y() <= parent_rect.min_y()
            {
                let js = document::eval(
                    r#"
let id = await dioxus.recv();
let elem = document.getElementById(id);
elem.scrollIntoView({ behavior: 'instant', block: 'nearest' });
                            "#,
                );
                js.send(item_id)?;
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Clone)]
pub struct CommandInput {
    pub value: String,
}

impl CommandInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

impl From<String> for CommandInput {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl Deref for CommandInput {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[component]
pub fn CommandList(items: ReadOnlySignal<Vec<CommandItem>>) -> Element {
    let command_input: Signal<CommandInput> = use_context();
    let command_result: Signal<CommandResult> = use_context();

    let mut focus_item_index = use_signal(|| 0);

    let matcher = use_context_provider(|| Arc::new(SkimMatcherV2::default().ignore_case()));

    let mut filtered_items = use_signal(Vec::new);

    let kvstore: kv::Store = use_context();

    let command_history = use_hook(|| {
        kvstore
            .bucket::<String, String>(Some("cmdpal.command_history"))
            .context("cmdpal.command_history")
    })?;

    use_effect({
        to_owned![command_history];
        move || {
            let items_len = items.len();
            filtered_items.set(
                items
                    .iter()
                    .filter_map(|cmd| {
                        let count = command_history
                            .get(&cmd.title)
                            .map(|v| v.and_then(|v| v.parse::<usize>().ok()).unwrap_or(0))
                            .unwrap_or(0);

                        let input = &command_input.read().value;

                        cmd.hints
                            .iter()
                            .chain([&cmd.title, &cmd.sub_title])
                            .find_map(|text| {
                                matcher.fuzzy_match(text, input).map(|score| {
                                    (
                                        (count + items_len) as i64 + score,
                                        CommandItem::from(cmd.clone()),
                                    )
                                })
                            })
                    })
                    .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
                    .map(|(_, cmd)| cmd)
                    .collect_vec(),
            );
        }
    });

    use_effect(move || {
        let len = filtered_items.len();
        let idx = *focus_item_index.peek();
        if idx >= len {
            focus_item_index.set(len.saturating_sub(1));
        }
    });

    let mut ctx = use_context_provider(|| CommandListCtx {
        node_ref: Signal::new(None),
        focus_item_index,
        filtered_items,
        command_result,

        command_history,
    });

    init_keybinding(ctx.clone());

    rsx! {
        div {
            onmounted: move |e| {
                ctx.node_ref.set(Some(e.data()));
            },
            class: "overflow-auto",
            div {
                class: "flex flex-col m-[12]",
                for (idx, item) in filtered_items.iter().enumerate() {
                    div {
                        class: "h-[56]",
                        CommandListItem {
                            idx,
                            item: item.clone(),
                            onclick: move |_| {
                                focus_item_index.set(idx)
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandListItem(
    idx: usize,
    item: CommandItem,
    #[props(default)] onclick: EventHandler<()>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut command_result: Signal<CommandResult> = use_context();
    let ctx: CommandListCtx = use_context();

    let mut node_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    let id = use_unique_id();

    use_effect({
        to_owned![ctx];
        move || {
            if ctx.is_selected(idx) {
                if let Some(node_ref) = node_ref() {
                    to_owned![ctx, id];
                    spawn(async move {
                        if let Err(e) = ctx.try_scroll_to_item(id(), node_ref).await {
                            warn!("{:?}", e);
                        }
                    });
                }
            }
        }
    });

    rsx! {
        div {
            id,
            class: "w-full h-full flex flex-row items-center rounded-sm outline-none",
            class: if ctx.is_selected(idx) {
                "bg-base-content/10"
            },
            onmounted: move |e: Event<MountedData>| async move {
                node_ref.set(Some(e.data()));
            },
            onclick: move |_| {
                onclick.call(());
            },
            ondoubleclick: move |_| {
                to_owned![item.command];
                async move {
                    command_result.set(command.invoke().await.unwrap());
                }
            },
            div {
                class: "size-[20] m-[16] bg-contain bg-center shrink-0",
                background_image: if let Some(icon) = item.icon { icon }
            },
            div {
                class: "inline-flex flex-col overflow-hidden",
                span {
                    class: "text-base truncate",
                    "{item.title}",
                },
                span {
                    class: "text-xs text-[#909090] truncate",
                    "{item.sub_title}",
                }
            }
        }
    }
}

fn init_keybinding(ctx: CommandListCtx) {
    let keybinding = use_context::<Keybinding>();

    let invoke_command = {
        to_owned![ctx];

        let mut command_input: Signal<CommandInput> = use_context();

        use_callback(move |_| -> Result<(), anyhow::Error> {
            let CommandListCtx {
                filtered_items,
                focus_item_index,
                mut command_result,
                command_history,
                ..
            } = ctx.clone();

            if let Some(cmd) = filtered_items.read().get(focus_item_index()).cloned() {
                let value = if let Some(record) = command_history.get(&cmd.title)? {
                    let count: usize = record.parse()?;
                    (count + 1).to_string()
                } else {
                    "0".to_string()
                };

                command_history.set(&cmd.title, &value)?;

                spawn(async move {
                    if let Err(e) = command_history.flush_async().await {
                        warn!("{:?}", e);
                    }

                    match cmd.command.invoke().await {
                        Ok(result) => {
                            command_input.set(CommandInput::new());

                            command_result.set(result);
                        }
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            Ok(())
        })
    };

    let next_item = {
        to_owned![ctx];
        use_callback(move |_| -> Result<(), anyhow::Error> {
            let len = ctx.filtered_items.len();
            let mut cur = ctx.focus_item_index.write();
            if *cur + 1 >= len {
                *cur = 0
            } else {
                *cur += 1
            }
            Ok(())
        })
    };

    let prev_item = {
        to_owned![ctx];
        use_callback(move |_| -> Result<(), anyhow::Error> {
            let len = ctx.filtered_items.len();
            let mut cur = ctx.focus_item_index.write();

            if *cur == 0 {
                *cur = len - 1
            } else {
                *cur -= 1
            }
            Ok(())
        })
    };

    use_hook_with_cleanup(
        move || {
            let name = format!("cmdpal.command_list.{}", random::<usize>());
            let km = generate_keymap!(
                ("<Return>", local_action!(invoke_command)),
                ("C-n", local_action!(next_item)),
                ("C-p", local_action!(prev_item)),
            )
            .unwrap();

            keybinding.push_keymap(&name, km);
            (name, keybinding)
        },
        move |(name, keybinding)| {
            keybinding.remove_keymap(&name);
        },
    );
}
