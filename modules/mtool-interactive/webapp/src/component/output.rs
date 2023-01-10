use gloo_console::debug;
use mtool_interactive_model::OutputContent;
use yew::prelude::*;

use crate::{keybinding::Keybinging, tauri, AppContext};

pub struct Output {
    keybinding: Keybinging,
    content: OutputContent,
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    Content(OutputContent),
}

impl Component for Output {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        ctx.link().send_future(async move {
            debug!("update view");
            Msg::Content(
                tauri::invoke("plugin:output|current_content", &())
                    .await
                    .unwrap(),
            )
        });

        let self_ = Self {
            keybinding: message.keybinding,
            content: OutputContent::None,
        };

        self_.register_keybinding(ctx);

        self_
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Content(c) => {
                self.content = c;
                true
            }
            Msg::AppContext(_) => unreachable!(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.content {
            OutputContent::Plain(text) => {
                html! {
                    <div class={classes!("output-plain")}>
                    { html_escape::encode_text(text).to_string() }
                    </div>
                }
            }
            OutputContent::None => html! {
                <div> { "No Content" } </div>
            },
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.unregister_keybinding();
    }
}

impl Output {
    fn register_keybinding(&self, ctx: &Context<Self>) {
        let _send = |msg: Msg| {
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
        // self.keybinding
        //     .define("C-a", send(Msg::MoveToLineBegin))
        //     .unwrap();
        // self.keybinding
        //     .define("C-e", send(Msg::MoveToLineEnd))
        //     .unwrap();
        // self.keybinding
        //     .define("C-f", send(Msg::ForwardChar))
        //     .unwrap();
        // self.keybinding
        //     .define("C-b", send(Msg::BackwardChar))
        //     .unwrap();
        // self.keybinding.define("C-k", send(Msg::Kill)).unwrap();
    }

    fn unregister_keybinding(&self) {
        // self.keybinding.remove("C-a").unwrap();
        // self.keybinding.remove("C-e").unwrap();
        // self.keybinding.remove("C-f").unwrap();
        // self.keybinding.remove("C-b").unwrap();
        // self.keybinding.remove("C-k").unwrap();
    }
}
