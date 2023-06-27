use std::rc::Rc;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use mtool_wgui::{
    generate_keymap, AppStage, AutoResizeWindow, Horizontal, Keybinding, RouteParams, Router,
    Vertical, WindowProps,
};
use serde::Serialize;
use tracing::debug;
use web_sys::{HtmlElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_icons::{Icon, IconId};

use crate::translator::{Backend, LanguageType};

pub struct App {
    keybinding: Keybinding,
    root: NodeRef,
    editor: NodeRef,
    backend: Backend,
    source: LanguageType,
    target: LanguageType,
    result: Option<Result<String, Rc<serde_wasm_bindgen::Error>>>,
}

#[derive(Clone)]
pub enum AppMsg {
    ToTarget(LanguageType),
    UseBackend(Backend),
    Translate,
    ShowTranslate(Result<String, Rc<serde_wasm_bindgen::Error>>),
}

impl Component for App {
    type Message = AppMsg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            keybinding: Keybinding::new(),
            root: NodeRef::default(),
            editor: NodeRef::default(),
            backend: Backend::Openai,
            source: LanguageType::Auto,
            target: LanguageType::En,
            result: Some(Ok("".into())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::ToTarget(target) => {
                self.target = target;
                true
            }
            AppMsg::UseBackend(backend) => {
                self.backend = backend;
                true
            }
            AppMsg::Translate => {
                self.result = None;
                self.translate(ctx);
                true
            }
            AppMsg::ShowTranslate(result) => {
                self.result = Some(result);
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
                                     "text-base",
                                     "font-mono")}
                     tabindex=0
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
                    class={classes!("flex",
                                    "h-40")}>
                    if let Some(result) = self.result.as_ref() {
                        <div>{ Self::render_translate_content(result) }</div>
                    } else {
                        <Icon
                            class={classes!("animate-spin",
                                            "m-1")}
                            icon_id={IconId::FontAwesomeSolidCircleNotch}
                            width={"1em".to_owned()}
                            height={"1em".to_owned()}/>
                    }
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
    fn translate(&mut self, ctx: &Context<Self>) {
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
            AppMsg::ShowTranslate(
                mtauri_sys::invoke("plugin:translate|text_translate", &args)
                    .await
                    .map_err(|e| Rc::new(e)),
            )
        })
    }

    fn render_translate_content(result: &Result<String, Rc<serde_wasm_bindgen::Error>>) -> Html {
        match result {
            Ok(result) => html! {
                { result.clone() }
            },
            Err(e) => html! {
                { e.to_string() }
            },
        }
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
            ("C-e", send(AppMsg::ToTarget(LanguageType::En))),
            ("C-z", send(AppMsg::ToTarget(LanguageType::Zh))),
            ("C-j", send(AppMsg::ToTarget(LanguageType::Ja))),
            ("C-A-o", send(AppMsg::UseBackend(Backend::Openai))),
            ("C-A-l", send(AppMsg::UseBackend(Backend::Llama))),
            ("C-A-t", send(AppMsg::UseBackend(Backend::Tencent))),
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
