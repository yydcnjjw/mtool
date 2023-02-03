use gloo_console::debug;
use mkeybinding::KeyMap;
use serde::{Deserialize, Serialize};
use web_sys::HtmlElement;
use yew::{platform::spawn_local, prelude::*, suspense::use_future_with_deps};

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
pub struct BaseProps {
    pub id: String,
    pub items: Vec<String>,
}

pub struct BaseCompletionList {
    item_nodes: Vec<NodeRef>,
    focus_item: usize,
    keybinding: Keybinging,
    km: KeyMap<SharedAction>,
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    Next,
    Prev,
    FocusChanged(usize),
    Exit,
}

impl BaseCompletionList {
    const COMPLETION_LIST_KEYMAP: &str = "completion_list";
}

impl Component for BaseCompletionList {
    type Message = Msg;

    type Properties = BaseProps;

    fn create(ctx: &Context<Self>) -> Self {
        debug!("BaseCompletionList()");

        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        let mut item_nodes = Vec::new();
        item_nodes.resize(ctx.props().items.len(), NodeRef::default());

        let self_ = Self {
            item_nodes,
            focus_item: 0,
            keybinding: message.keybinding,
            km: Self::generate_keymap(ctx),
        };

        Self::adjust_window_size(ctx.props(), None);

        self_
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Next => {
                let index = if self.focus_item == ctx.props().items.len() - 1 {
                    0
                } else {
                    self.focus_item + 1
                };

                self.focus_item = index;

                true
            }
            Msg::Prev => {
                let index = if self.focus_item == 0 {
                    ctx.props().items.len() - 1
                } else {
                    self.focus_item - 1
                };

                self.focus_item = index;

                true
            }
            Msg::Exit => {
                let completed = ctx.props().items[self.focus_item].to_owned();
                spawn_local(async move {
                    let _: () = tauri::invoke(
                        "plugin:completion|complete_exit",
                        &CompletionArgs { completed },
                    )
                    .await
                    .unwrap();
                });
                false
            }
            Msg::FocusChanged(index) => {
                self.focus_item = index;
                false
            }
            Msg::AppContext(_) => {
                unreachable!();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("completion-list")}>
            {
                ctx.props().items.iter().enumerate().map(|(i, item)|{
                    html!{
                      <div ref={&self.item_nodes[i]}
                        class={classes!("completion-item")}
                        tabIndex="1"
                        onfocus={ctx.link().callback(move |_| Msg::FocusChanged(i))}
                        onmouseover={ctx.link().callback(move |_| Msg::FocusChanged(i))}
                        >
                        { item }
                      </div>
                    }
                }).collect::<Html>()
            }
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        debug!("CompletionList changed");

        let items = &ctx.props().items;
        self.item_nodes.resize(items.len(), NodeRef::default());
        self.focus_item = 0;

        self.remap_keymap(ctx);

        Self::adjust_window_size(ctx.props(), Some(old_props));

        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render && !ctx.props().items.is_empty() {
            self.keybinding
                .push_keymap(Self::COMPLETION_LIST_KEYMAP, self.km.clone());
        }

        debug!(self.focus_item);

        let elm = self.item_nodes[self.focus_item]
            .cast::<HtmlElement>()
            .unwrap();
        debug!(&elm);
        elm.focus().unwrap();
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.keybinding.remove_keymap(Self::COMPLETION_LIST_KEYMAP);
    }
}

impl BaseCompletionList {
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

    fn remap_keymap(&mut self, ctx: &Context<Self>) {
        let items = &ctx.props().items;
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

    fn adjust_window_size(new_props: &BaseProps, old_props: Option<&BaseProps>) {
        const MAX_ITEM_COUNT: usize = 5;

        let need_adjust = if let Some(old_props) = old_props {
            new_props.id != old_props.id
                || new_props.items.len().min(MAX_ITEM_COUNT)
                    != old_props.items.len().min(MAX_ITEM_COUNT)
        } else {
            true
        };

        if !need_adjust {
            return;
        }

        let visual_item_count = new_props.items.len().min(MAX_ITEM_COUNT);

        let width = 720;

        let height = if visual_item_count == 0 {
            50
        } else {
            visual_item_count * 48 + 2 + 50 + 16
        };

        spawn_local(window::set_size(window::PhysicalSize { width, height }));
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub id: String,
    pub input: String,
}

#[function_component]
pub fn CompletionList(props: &Props) -> HtmlResult {
    let items = use_future_with_deps(
        |props| async move {
            let items: Vec<String> = tauri::invoke(
                "plugin:completion|complete_read",
                &CompletionArgs {
                    completed: props.input.to_string(),
                },
            )
            .await
            .unwrap();
            items
        },
        props.clone(),
    )?;

    Ok(html! {
        <BaseCompletionList id={props.id.clone()} items={(*items).clone()} />
    })
}
