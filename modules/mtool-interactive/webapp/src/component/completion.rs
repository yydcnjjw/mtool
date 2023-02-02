use gloo_console::debug;
use mtool_interactive_model::CompletionMeta;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

use crate::{
    generate_keymap,
    keybinding::Keybinging,
    tauri::self,
    AppContext,
};

use super::completion_list::{CompletionArgs, CompletionList};

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

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

impl Component for Completion {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        debug!("Completion()");

        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        let mut self_ = Self {
            input: String::default(),
            input_node: NodeRef::default(),
            keybinding: message.keybinding,
            meta: CompletionMeta::default(),
        };

        Self::refresh(&mut self_, ctx);

        self_.register_keybinding(ctx);

        self_
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        debug!("Completion changed");
        self.refresh(ctx);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Input(_) => {
                self.input = self.input_node().unwrap().value();
                true
            }
            Msg::ForwardChar => {
                let input_node = self.input_node().unwrap();

                let start = input_node.selection_start().unwrap().unwrap();

                let start = if start == self.input.len() as u32 {
                    start
                } else {
                    start + 1
                };

                input_node.set_selection_range(start, start).unwrap();
                false
            }
            Msg::BackwardChar => {
                let input_node = self.input_node().unwrap();

                let start = input_node.selection_start().unwrap().unwrap();
                let start = if start == 0 { 0 } else { start - 1 };

                input_node.set_selection_range(start, start).unwrap();
                false
            }
            Msg::MoveToLineBegin => {
                self.input_node()
                    .unwrap()
                    .set_selection_range(0, 0)
                    .unwrap();
                false
            }
            Msg::MoveToLineEnd => {
                let input = self.input_node().unwrap();
                let end = input.value().len() as u32;
                input.set_selection_range(end, end).unwrap();
                false
            }
            Msg::Kill => {
                let input_node = self.input_node().unwrap();
                let start = input_node.selection_start().unwrap().unwrap() as usize;

                let input = &self.input[0..start];
                input_node.set_value(input);
                ctx.link().send_message(Msg::Input(input.to_string()));

                // TODO: copy value
                false
            }
            Msg::Exit => {
                let completed = self.input.clone();

                self.clear_input();

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
                  <input ref={self.input_node.clone()}
                  {oninput}
                  class={classes!("input")}
                  type="text"
                  placeholder={self.meta.prompt.clone()}
                  autofocus=true/>
                </div>

                <div class={classes!("content-box")}>
                  <Suspense {fallback}>
                  <CompletionList
                    id={ctx.props().id.clone()} input={self.input.clone()}/>
                  </Suspense>
                </div>

            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.input_node().unwrap().focus().unwrap();
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.unregister_keybinding();
    }
}

impl Completion {
    fn refresh(&mut self, ctx: &Context<Self>) {
        Self::fetch_completion_meta(ctx);
        self.clear_input();
    }

    fn clear_input(&mut self) {
        if let Some(input) = self.input_node() {
            input.set_value("");
        }
        self.input.clear();
    }

    fn fetch_completion_meta(ctx: &Context<Self>) {
        ctx.link().send_future(async move {
            Msg::CompletionMeta(
                tauri::invoke("plugin:completion|completion_meta", &())
                    .await
                    .unwrap(),
            )
        });
    }

    fn input_node(&self) -> Option<HtmlInputElement> {
        self.input_node.cast::<HtmlInputElement>()
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

        let km = generate_keymap!(
            ("C-a", send(Msg::MoveToLineBegin)),
            ("C-e", send(Msg::MoveToLineEnd)),
            ("C-f", send(Msg::ForwardChar)),
            ("C-b", send(Msg::BackwardChar)),
            ("C-k", send(Msg::Kill)),
            ("<Return>", send(Msg::Exit)),
        )
        .unwrap();

        self.keybinding.push_keymap("completion", km);
    }

    fn unregister_keybinding(&self) {
        self.keybinding.pop_keymap();
    }
}
