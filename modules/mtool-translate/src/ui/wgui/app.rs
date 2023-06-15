use async_trait::async_trait;
use mtool_wgui::{
    generate_keymap, AppStage, AutoResizeWindow, Horizontal, Keybinging, RouteParams, Router,
    Vertical, WindowProps,
};
use serde::Serialize;
use tracing::{debug, warn};
use web_sys::{HtmlElement, HtmlTextAreaElement};
use yew::prelude::*;
// use yew_icons::{Icon, IconId};
use mapp::{provider::Res, AppContext, AppModule};

use crate::translator::{Backend, LanguageType};

pub struct App {
    keybinding: Keybinging,
    root: NodeRef,
    editor: NodeRef,
    backend: Backend,
    source: LanguageType,
    target: LanguageType,
    result: String,
}

// #[derive(Properties, PartialEq)]
// pub struct AppProps {
//     path: String,
// }

#[derive(Clone)]
pub enum AppMsg {
    ToEnglish,
    ToChinese,
    ToJapanese,
    UseOpenai,
    UseLLama,
    UseTencent,
    Translate,
    ShowTranslate(String),
}

impl Component for App {
    type Message = AppMsg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            keybinding: Keybinging::new(),
            root: NodeRef::default(),
            editor: NodeRef::default(),
            backend: Backend::Openai,
            source: LanguageType::Auto,
            target: LanguageType::En,
            result: String::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::ToEnglish => {
                self.target = LanguageType::En;
                true
            }
            AppMsg::ToChinese => {
                self.target = LanguageType::Zh;
                true
            }
            AppMsg::ToJapanese => {
                self.target = LanguageType::Ja;
                true
            }
            AppMsg::Translate => {
                self.translate(ctx);
                true
            }
            AppMsg::ShowTranslate(result) => {
                self.result = result;
                true
            }
            AppMsg::UseOpenai => {
                self.backend = Backend::Openai;
                true
            }
            AppMsg::UseLLama => {
                self.backend = Backend::Llama;
                true
            }
            AppMsg::UseTencent => {
                self.backend = Backend::Tencent;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
            <AutoResizeWindow window={
                WindowProps{
                    horizontal: Horizontal::Center,
                    vertical: Vertical::Absolute(350),
                    ..Default::default()
                }
            }>
                <div class={classes!("flex",
                                     "flex-col",
                                     "divide-y",
                                     "divide-gray-600",
                                     "p-2",
                                     "rounded-md",
                                     "bg-black",
                                     "text-white",
                                     "text-base")}
                     tabindex="0"
                     ref={ self.root.clone() }>
                  <div class={classes!("columns-3","m-1")}>
                    <div>
                      <span>{ "source: " }</span>
                      <span>{ self.source.clone() }</span>
                    </div>
                    <div>
                      <span>{ "target: " }</span>
                      <span>{ self.target.clone() }</span>
                    </div>
                    <div>
                      <span>{ "backend: " }</span>
                      <span>{ self.backend.clone() }</span>
                    </div>
                  </div>
                  <textarea
                    class={classes!("resize-none",
                                    "bg-black",
                                    "text-white",
                                    "outline-none")}
                    ref={ self.editor.clone() }
                    rows="5"
                    cols="50"
                    placeholder="Input text">
                   </textarea>
                  <div
                    class={classes!("h-40")}>
                    { self.result.clone() }
                  </div>
                </div>
            </AutoResizeWindow>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.register_keybinding(ctx);
            self.editor.cast::<HtmlElement>().unwrap().focus().unwrap();
        }
    }
}

impl App {
    fn translate(&self, ctx: &Context<Self>) {
        #[derive(Debug, Serialize)]
        struct TranslateArgs {
            input: String,
            source: LanguageType,
            target: LanguageType,
            backend: Backend,
        }
        let args = TranslateArgs {
            input: self.editor.cast::<HtmlTextAreaElement>().unwrap().value(),
            source: self.source.clone(),
            target: self.target.clone(),
            backend: self.backend.clone(),
        };
        ctx.link().send_future(async move {
            match mtauri_sys::invoke("plugin:translate|text_translate", &args).await {
                Ok(result) => AppMsg::ShowTranslate(result),
                Err(e) => {
                    warn!("invoke text_translate failed: {:?}", e);
                    AppMsg::ShowTranslate("error occurred".into())
                }
            }
        })
    }

    fn register_keybinding(&self, ctx: &Context<Self>) {
        debug!("register_keybinding");
        let send = |msg: AppMsg| {
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
            ("C-e", send(AppMsg::ToEnglish)),
            ("C-z", send(AppMsg::ToChinese)),
            ("C-j", send(AppMsg::ToJapanese)),
            ("C-A-o", send(AppMsg::UseOpenai)),
            ("C-A-l", send(AppMsg::UseLLama)),
            ("C-A-t", send(AppMsg::UseTencent)),
            ("C-<Return>", send(AppMsg::Translate)),
        )
        .unwrap();

        self.keybinding.push_keymap("translate", km);

        self.keybinding.setup_on_keydown(|f| {
            let root = self.root.cast::<HtmlElement>().unwrap();
            root.set_onkeydown(f);
        });
    }
}

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(AppStage::Init, init);
        Ok(())
    }
}

fn render(_: &RouteParams) -> Html {
    html! {
        <App/>
    }
}

async fn init(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/translate", render);
    Ok(())
}
