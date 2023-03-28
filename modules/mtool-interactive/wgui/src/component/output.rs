use mtool_interactive_model::OutputContent;
use tracing::debug;
use yew::{platform::spawn_local, prelude::*};

use crate::{app::AppContext, keybinding::Keybinging};

pub struct Output {
    #[allow(dead_code)]
    keybinding: Keybinging,
    content: OutputContent,
}

#[derive(Clone)]
pub enum Msg {
    AppContext(AppContext),
    Content(OutputContent),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

impl Component for Output {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        debug!("Output()");
        let (message, _) = ctx
            .link()
            .context(ctx.link().callback(Msg::AppContext))
            .expect("No AppContext Provided");

        let self_ = Self {
            keybinding: message.keybinding,
            content: OutputContent::None,
        };

        Self::refresh(ctx);

        self_.register_keybinding(ctx);

        self_
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        debug!("Output changed");
        Self::refresh(ctx);
        true
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
                    <div class={ classes!("output") }>
                      <div class={ classes!("output-plain") }>
                      { html_escape::encode_text(text).to_string() }
                      </div>
                    </div>
                }
            }
            OutputContent::None => html! {
                <div class={ classes!("output") }> { "Loading" } </div>
            },
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.unregister_keybinding();
    }
}

impl Output {
    fn refresh(ctx: &Context<Self>) {
        ctx.link().send_future(async move {
            Msg::Content(
                mtauri_sys::invoke("plugin:interactive::output|current_content", &())
                    .await
                    .unwrap(),
            )
        });

        Self::adjust_window_size();
    }

    fn adjust_window_size() {
        spawn_local(mtauri_sys::window::set_size(
            mtauri_sys::window::PhysicalSize {
                width: 720,
                height: 480,
            },
        ));
    }

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
