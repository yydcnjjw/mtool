use mtool_wgui::{AutoResizeWindow, Horizontal, Vertical, WindowProps};
use yew::prelude::*;

use crate::ui::wgui::model::OutputContent;

pub struct Output {
    content: OutputContent,
}

#[derive(Clone)]
pub enum Msg {
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
        let self_ = Self {
            content: OutputContent::None,
        };

        Self::refresh(ctx);

        self_
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        Self::refresh(ctx);
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Content(c) => {
                self.content = c;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.content {
            OutputContent::Plain(text) => {
                html! {
                    <AutoResizeWindow
                      window={
                          WindowProps{
                              horizontal: Horizontal::Center,
                              vertical: Vertical::Absolute(350),
                              ..Default::default()}}>
                    <div class={classes!(
                        "bg-black",
                        "w-[32rem]",
                        "h-[32rem]",
                        "text-white",
                        "rounded-xl",
                        "overflow-hidden")}>
                      <div class={classes!("w-full", "h-full")}>
                      { html_escape::encode_text(text).to_string() }
                      </div>
                    </div>
                    </AutoResizeWindow>
                }
            }
            OutputContent::None => html! {
                <div class={classes!("text-xl")}> { "Loading" } </div>
            },
        }
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
    }
}
