use serde::{Deserialize, Serialize};
use yew::{platform::spawn_local, prelude::*, suspense::use_future_with_deps};

use crate::{keybinding::Keybinging, tauri, AppContext};

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
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    Next,
    Prev,
    Exit,
}

impl Component for BaseCompletionList {
    type Message = Msg;

    type Properties = BaseProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        let self_ = Self {
            focus_item: 0,
            keybinding: message.keybinding,
        };

        self_.register_keybinding(ctx);

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

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.focus_item = 0;
        true
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.unregister_keybinding();
    }
}

impl BaseCompletionList {
    fn register_keybinding(&self, ctx: &Context<Self>) {
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
        self.keybinding.define("C-n", send(Msg::Next)).unwrap();
        self.keybinding.define("C-p", send(Msg::Prev)).unwrap();
        self.keybinding.define("<Return>", send(Msg::Exit)).unwrap();
    }

    fn unregister_keybinding(&self) {
        self.keybinding.remove("C-n").unwrap();
        self.keybinding.remove("C-p").unwrap();
        self.keybinding.remove("<Return>").unwrap();
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub input: String,
}

#[function_component]
pub fn CompletionList(props: &Props) -> HtmlResult {
    let items = use_future_with_deps(
        |input| async move {
            let items: Vec<String> = tauri::invoke(
                "plugin:completion|complete_read",
                &CompletionArgs {
                    completed: input.to_string(),
                },
            )
            .await
            .unwrap();
            items
        },
        props.input.clone(),
    )?;

    Ok(html! {<BaseCompletionList items={(*items).clone()} />})
}
