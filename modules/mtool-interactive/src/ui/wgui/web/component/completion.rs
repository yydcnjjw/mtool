use mtool_wgui::{
    generate_keymap, AutoResizeWindow, Horizontal, Keybinging, Vertical, WindowProps,
};
use tracing::warn;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::{platform::spawn_local, prelude::*};

use crate::{completion::CompletionMeta, ui::wgui::model::CompletionExit};

use super::completion_list::{CompletionExitArgs, CompletionList};

pub struct Completion {
    root_node: NodeRef,
    input: String,
    input_node: NodeRef,
    keybinding: Keybinging,
    meta: CompletionMeta,
}

#[derive(Clone)]
pub enum Msg {
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
        let mut self_ = Self {
            root_node: NodeRef::default(),
            input: String::default(),
            input_node: NodeRef::default(),
            keybinding: Keybinging::new(),
            meta: CompletionMeta::default(),
        };

        Self::refresh(&mut self_, ctx);

        self_
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
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
                    if let Err(e) = mtauri_sys::invoke::<CompletionExitArgs, ()>(
                        "plugin:interactive::completion|complete_exit",
                        &CompletionExitArgs {
                            v: CompletionExit::Completed(completed),
                        },
                    )
                    .await
                    {
                        warn!("invoke complete_exit failed: {:?}", e);
                    }
                });

                false
            }
            Msg::CompletionMeta(meta) => {
                self.meta = meta;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx
            .link()
            .callback(move |e: InputEvent| Msg::Input(e.data().unwrap_or_default()));

        html! {
            <>
                <AutoResizeWindow
                  window={
                      WindowProps{
                          horizontal: Horizontal::Center,
                          vertical: Vertical::Absolute(350),
                          ..Default::default()}}>
                <div class={classes!("w-[48rem]",
                                     "text-white")}
                     ref={ self.root_node.clone() }>
                  <div class={classes!("flex",
                                     "w-full")}>
                    <input ref={ self.input_node.clone() }
                      class={classes!("w-full",
                                    "h-16",
                                    "rounded-xl",
                                    "overflow-hidden",
                                    "text-3xl",
                                    "bg-black",
                                    "appearance-none",
                                    "caret-white",
                                    "px-4",
                                    "font-mono",
                                    "outline-none")}
                      {oninput}
                      type="text"
                      placeholder={self.meta.prompt.clone()}
                      autofocus=true/>
                  </div>

                  <div class={classes!("w-full", "h-2", "bg-transparent")} />

                  <CompletionList
                    class={classes!("flex",
                                  "flex-col",
                                  "bg-black",
                                  "rounded-xl",
                                  "overflow-hidden")}
                    id={ctx.props().id.clone()}
                    input={self.input.clone()}
                    keybinding={self.keybinding.clone()}/>
                </div>
                </AutoResizeWindow>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.register_keybinding(ctx);
            self.input_node().unwrap().focus().unwrap();
        }
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
                mtauri_sys::invoke("plugin:interactive::completion|completion_meta", &())
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

        self.keybinding.setup_on_keydown(|f| {
            self.root_node
                .cast::<HtmlElement>()
                .unwrap()
                .set_onkeydown(f);
        })
    }
}
