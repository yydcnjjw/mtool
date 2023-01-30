use mtool_interactive_model::CompletionMeta;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

use crate::{keybinding::Keybinging, tauri, AppContext};

use super::completion_list::{CompletionList, CompletionArgs};

pub struct Completion {
    input: String,
    input_node: NodeRef,
    keybinding: Keybinging,
    meta: CompletionMeta,
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    Input(String),
    CompletionMeta(CompletionMeta),
    ForwardChar,
    BackwardChar,
    MoveToLineBegin,
    MoveToLineEnd,
    Kill,
    Exit,
}

impl Component for Completion {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        ctx.link().send_future(async move {
            Msg::CompletionMeta(
                tauri::invoke("plugin:completion|completion_meta", &())
                    .await
                    .unwrap(),
            )
        });

        let self_ = Self {
            input: Default::default(),
            input_node: NodeRef::default(),
            keybinding: message.keybinding,
            meta: CompletionMeta::default(),
        };

        self_.register_keybinding(ctx);

        self_
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Input(_) => {
                self.input = self.input_node().value();
                true
            }
            Msg::ForwardChar => {
                let input = self.input_node();

                let start = input.selection_start().unwrap().unwrap();

                let start = if start == self.input.len() as u32 {
                    start
                } else {
                    start + 1
                };

                self.input_node().set_selection_range(start, start).unwrap();
                false
            }
            Msg::BackwardChar => {
                let input = self.input_node();

                let start = input.selection_start().unwrap().unwrap();
                let start = if start == 0 { 0 } else { start - 1 };

                self.input_node().set_selection_range(start, start).unwrap();
                false
            }
            Msg::MoveToLineBegin => {
                self.input_node().set_selection_range(0, 0).unwrap();
                false
            }
            Msg::MoveToLineEnd => {
                let input = self.input_node();
                let end = input.value().len() as u32;
                input.set_selection_range(end, end).unwrap();
                false
            }
            Msg::Kill => {
                let start = self.input_node().selection_start().unwrap().unwrap() as usize;

                let input = &self.input[0..start];
                self.input_node().set_value(input);
                ctx.link().send_message(Msg::Input(input.to_string()));

                // TODO: copy value
                false
            }
            Msg::Exit => {
                let completed = self.input.clone();
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
            Msg::CompletionMeta(meta) => {
                self.meta = meta;
                true
            }
            Msg::AppContext(_) => unreachable!(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx
            .link()
            .callback(move |e: InputEvent| Msg::Input(e.data().unwrap_or_default()));

        let fallback = html! { <div>{ "Loading..." }</div> };

        html! {
            <div class={classes!("completion")}>
                <div class={classes!("search-box")}>
                  if {!self.meta.prompt.is_empty()} {
                      <div class={classes!("prompt")}>{self.meta.prompt.clone()}</div>
                  }

                  <input ref={self.input_node.clone()}
                  {oninput}
                  class={classes!("input")}
                  type="text"
                  autofocus=true/>
                </div>
                <Suspense {fallback}>
                <CompletionList input={self.input.clone()}/>
                </Suspense>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.input_node().focus().unwrap();
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.unregister_keybinding();
    }
}

impl Completion {
    fn input_node(&self) -> HtmlInputElement {
        self.input_node.cast::<HtmlInputElement>().unwrap()
    }

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
        self.keybinding
            .define("C-a", send(Msg::MoveToLineBegin))
            .unwrap();
        self.keybinding
            .define("C-e", send(Msg::MoveToLineEnd))
            .unwrap();
        self.keybinding
            .define("C-f", send(Msg::ForwardChar))
            .unwrap();
        self.keybinding
            .define("C-b", send(Msg::BackwardChar))
            .unwrap();
        self.keybinding.define("C-k", send(Msg::Kill)).unwrap();
        self.keybinding.define("C-A-j", send(Msg::Exit)).unwrap();
    }

    fn unregister_keybinding(&self) {
        self.keybinding.remove("C-a").unwrap();
        self.keybinding.remove("C-e").unwrap();
        self.keybinding.remove("C-f").unwrap();
        self.keybinding.remove("C-b").unwrap();
        self.keybinding.remove("C-k").unwrap();
        self.keybinding.remove("C-A-j").unwrap();
    }
}
