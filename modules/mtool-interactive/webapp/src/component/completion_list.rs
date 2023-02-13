use gloo_console::debug;
use gloo_utils::document;
use mkeybinding::KeyMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, ScrollIntoViewOptions, ScrollLogicalPosition};
use yew::{platform::spawn_local, prelude::*};

use crate::{
    generate_keymap,
    keybinding::{Keybinging, SharedAction},
    tauri::{self, window},
    AppContext,
};

#[derive(Serialize, Deserialize)]
pub struct CompletionArgs {
    pub completed: String,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
    pub input: String,
}

pub struct CompletionList {
    items: Vec<String>,
    focused_item_index: usize,
    keybinding: Keybinging,
    km: KeyMap<SharedAction>,
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    FetchCompleteRead(Vec<String>),
    Next,
    Prev,
    FocusChanged(usize),
    Exit,
}

impl CompletionList {
    const COMPLETION_LIST_KEYMAP: &str = "completion_list";
}

impl Component for CompletionList {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        debug!("BaseCompletionList()");

        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        let self_ = Self {
            items: Vec::new(),
            focused_item_index: 0,
            keybinding: message.keybinding,
            km: Self::generate_keymap(ctx),
        };

        Self::fetch_complete_read(ctx);

        self_
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchCompleteRead(items) => {
                let need_adjust = self.items.len().min(Self::MAX_ITEM_COUNT)
                    != items.len().min(Self::MAX_ITEM_COUNT);

                self.items = items;

                self.remap_keymap();

                if need_adjust {
                    self.adjust_window_size();
                }

                true
            }
            Msg::Next => {
                let index = if self.focused_item_index == self.items.len() - 1 {
                    0
                } else {
                    self.focused_item_index + 1
                };

                self.focused_item_index = index;

                true
            }
            Msg::Prev => {
                let index = if self.focused_item_index == 0 {
                    self.items.len() - 1
                } else {
                    self.focused_item_index - 1
                };

                self.focused_item_index = index;

                true
            }
            Msg::Exit => {
                let completed = self.items[self.focused_item_index].to_owned();
                spawn_local(async move {
                    let _: () = tauri::invoke(
                        "plugin:interactive::completion|complete_exit",
                        &CompletionArgs { completed },
                    )
                    .await
                    .unwrap();
                });
                false
            }
            Msg::FocusChanged(index) => {
                self.focused_item_index = index;
                true
            }
            Msg::AppContext(_) => {
                unreachable!();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let focus_class = |i| {
            if self.focused_item_index == i {
                "focus-item"
            } else {
                ""
            }
        };

        html! {
            if !self.items.is_empty() {
                <div class={classes!("completion-list")}>
                {
                    for self.items.iter().enumerate().map(|(i, item)|{
                        html! {
                            <div id={ Self::completion_item_id(i) }
                             class={ classes!("completion-item", focus_class(i)) }
                             onclick={ ctx.link().callback(move |_| Msg::FocusChanged(i)) }
                             >
                            { item }
                            </div>
                        }
                    })
                }
                </div>
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        debug!("CompletionList changed");

        self.focused_item_index = 0;

        Self::fetch_complete_read(ctx);

        if old_props.id != ctx.props().id {
            self.adjust_window_size();
        }

        true
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render && !self.items.is_empty() {
            self.keybinding
                .push_keymap(Self::COMPLETION_LIST_KEYMAP, self.km.clone());
        }

        self.scroll_into_focused_item();
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.keybinding.remove_keymap(Self::COMPLETION_LIST_KEYMAP);
    }
}

impl CompletionList {
    const MAX_ITEM_COUNT: usize = 5;

    fn fetch_complete_read(ctx: &Context<Self>) {
        let input = ctx.props().input.to_string();
        ctx.link().send_future(async move {
            let items: Vec<String> = tauri::invoke(
                "plugin:interactive::completion|complete_read",
                &CompletionArgs { completed: input },
            )
            .await
            .unwrap();
            Msg::FetchCompleteRead(items)
        });
    }

    fn generate_keymap(ctx: &Context<Self>) -> KeyMap<SharedAction> {
        let send = |msg: Msg| {
            let link = ctx.link().clone();
            move || {
                let link = link.clone();
                let msg = msg.clone();
                async move {
                    link.send_message(msg);
                    Ok::<(), anyhow::Error>(())
                }
            }
        };
        generate_keymap!(
            ("C-n", send(Msg::Next)),
            ("C-p", send(Msg::Prev)),
            ("<Return>", send(Msg::Exit)),
        )
        .unwrap()
    }

    fn remap_keymap(&mut self) {
        let items = &self.items;
        if items.is_empty() {
            self.keybinding.remove_keymap(Self::COMPLETION_LIST_KEYMAP);
        } else {
            if !self
                .keybinding
                .contains_keymap(Self::COMPLETION_LIST_KEYMAP)
            {
                self.keybinding
                    .push_keymap(Self::COMPLETION_LIST_KEYMAP, self.km.clone());
            }
        }
    }

    fn adjust_window_size(&self) {
        let visual_item_count = self.items.len().min(Self::MAX_ITEM_COUNT);

        let width = 720;

        let height = if visual_item_count == 0 {
            50
        } else {
            visual_item_count * 48 + 2 + 50 + 16
        };

        spawn_local(window::set_size(window::PhysicalSize { width, height }));
    }

    fn scroll_into_focused_item(&self) {
        if let Some(elm) = self.focused_item() {
            debug!(&elm);
            let mut opt = ScrollIntoViewOptions::new();
            opt.block(ScrollLogicalPosition::Nearest);

            elm.scroll_into_view_with_scroll_into_view_options(&opt);
        }
    }

    fn completion_item_id(i: usize) -> String {
        format!("completion-item-{}", i)
    }

    fn focused_item(&self) -> Option<HtmlElement> {
        document()
            .get_element_by_id(&Self::completion_item_id(self.focused_item_index))
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    }
}
