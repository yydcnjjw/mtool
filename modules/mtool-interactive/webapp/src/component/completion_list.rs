use gloo_console::debug;
use mkeybinding::KeyMap;
use serde::{Deserialize, Serialize};
use yew::{platform::spawn_local, prelude::*, suspense::use_future_with_deps};

use crate::{
    generate_keymap,
    keybinding::{Keybinging, SharedAction},
    tauri, AppContext,
};

#[derive(Serialize, Deserialize)]
pub struct CompletionArgs {
    pub completed: String,
}

#[derive(Properties, PartialEq)]
pub struct BaseProps {
    pub items: Vec<String>,
}

pub struct BaseCompletionList {
    focus_item: usize,
    keybinding: Keybinging,
    km: KeyMap<SharedAction>,
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    Next,
    Prev,
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

        let self_ = Self {
            focus_item: 0,
            keybinding: message.keybinding,
            km: Self::generate_keymap(ctx),
        };

        self_
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Next => {
                if self.focus_item == ctx.props().items.len() - 1 {
                    self.focus_item = 0;
                } else {
                    self.focus_item += 1;
                }

                true
            }
            Msg::Prev => {
                if self.focus_item == 0 {
                    self.focus_item = ctx.props().items.len() - 1;
                } else {
                    self.focus_item -= 1;
                }

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
            Msg::AppContext(_) => {
                unreachable!();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ul class={classes!("completion-list")}>
            {
                ctx.props().items.iter().enumerate().map(|(i, item)|{html!{
                    <li class={classes!("item", (i == self.focus_item).then(|| Some("focus")) )}>
                    { item }
                    </li>
                }}).collect::<Html>()
            }
            </ul>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.focus_item = 0;

        if ctx.props().items.is_empty() {
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

        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render && !ctx.props().items.is_empty() {
            self.keybinding
                .push_keymap(Self::COMPLETION_LIST_KEYMAP, self.km.clone());
        }
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
        <BaseCompletionList items={(*items).clone()} />
    })
}
