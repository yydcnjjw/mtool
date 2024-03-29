use std::any::type_name;

use async_trait::async_trait;
use mapp::prelude::*;
use mtool_wgui::{
    component::error::error_view, generate_keymap, AutoWindow, EmptyView, Horizontal, Keybinding,
    RouteParams, Router, TemplateData, TemplateId, TemplateView, Vertical, WebStage, WindowProps,
};
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

use crate::dict::Backend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub template_id: TemplateId,
    pub data: TemplateData,
}

#[derive(Properties, PartialEq)]
pub struct AppProps {
    query: String,
}

pub struct App {
    input_node: NodeRef,
    keybinding: Keybinding,
    query: String,
    query_result: Option<Result<QueryResult, anyhow::Error>>,
    backend: Backend,
}

pub enum AppMsg {
    UseBackend(Backend),
    QueryDict(String),
    QueryDictFromInput,
    ShowDict(Result<QueryResult, anyhow::Error>),
}

impl Component for App {
    type Message = AppMsg;

    type Properties = AppProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link()
            .send_message(AppMsg::QueryDict(ctx.props().query.clone()));
        Self {
            input_node: NodeRef::default(),
            keybinding: Keybinding::new(),
            query: String::default(),
            query_result: Some(Ok(QueryResult {
                template_id: type_name::<EmptyView>().into(),
                data: serde_json::to_value(()).unwrap(),
            })),
            backend: Backend::ECDict,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        ctx.link()
            .send_message(AppMsg::QueryDict(ctx.props().query.clone()));
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::UseBackend(backend) => {
                self.backend = backend;
                true
            }
            AppMsg::QueryDict(query) => {
                self.query_dict(ctx, query);
                true
            }
            AppMsg::QueryDictFromInput => {
                self.query_dict(ctx, self.get_input());
                true
            }
            AppMsg::ShowDict(result) => {
                self.query_result = Some(result);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
            <AutoWindow window={
                WindowProps{
                    horizontal: Horizontal::Center,
                    vertical: Vertical::Absolute(350),
                    ..Default::default()
                }
            }>
              <div class={classes!("flex",
                                   "flex-col",
                                   "w-[32rem]",
                                   "divide-y",
                                   "divide-gray-600",
                                   "p-2",
                                   "rounded-md",
                                   "bg-black",
                                   "text-white",
                                   "text-base",
                                   "font-mono")}>
                <div class={classes!("columns-1","m-1")}>
                  <div>
                    <span>{ "backend: " }</span>
                    <span>{ self.backend.to_string() }</span>
                  </div>
                </div>
                <input ref={ self.input_node.clone() }
                  class={classes!("w-full",
                                  "h-12",
                                  "overflow-hidden",
                                  "text-2xl",
                                  "bg-black",
                                  "appearance-none",
                                  "caret-white",
                                  "outline-none")}
                  type="text"
                  placeholder="Input text..."
                  autofocus=true
                  value={ self.query.clone() }/>
                if let Some(result) = self.query_result.as_ref() {
                    <div class={classes!("h-[16rem]",
                                         "w-full",)}>
                      { Self::render_dict_content(result) }
                    </div>
                } else {
                  <div>
                    <Icon
                      class={classes!("animate-spin",
                                      "m-1")}
                      icon_id={IconId::FontAwesomeSolidCircleNotch}
                      width={"1em".to_owned()}
                      height={"1em".to_owned()}/>
                  </div>
                }
              </div>
            </AutoWindow>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.register_keybinding(ctx);
            self.input_node
                .cast::<HtmlInputElement>()
                .unwrap()
                .focus()
                .unwrap();
        }
    }
}

impl App {
    fn get_input(&self) -> String {
        self.input_node.cast::<HtmlInputElement>().unwrap().value()
    }

    fn query_dict(&mut self, ctx: &Context<Self>, query: String) {
        #[derive(Debug, Serialize)]
        struct Args {
            query: String,
            backend: Backend,
        }

        self.query = query;

        if self.query.is_empty() {
            return;
        }

        if self.query_result.is_none() {
            return;
        }

        self.query_result = None;
        let args = Args {
            query: self.query.clone(),
            backend: self.backend.clone(),
        };
        ctx.link().send_future(async move {
            AppMsg::ShowDict(mtauri_sys::invoke("plugin:mtool-dict|dict_query", &args).await)
        })
    }

    fn render_dict_content(result: &Result<QueryResult, anyhow::Error>) -> Html {
        match result {
            Ok(QueryResult { template_id, data }) => html! {
                <div class={classes!("overflow-y-auto")}>
                  <TemplateView template_id={ template_id.clone() }
                                data={ data.clone() }/>
                </div>
            },
            Err(e) => error_view(e),
        }
    }

    fn register_keybinding(&self, ctx: &Context<Self>) {
        let send = |msg: fn() -> AppMsg| {
            let link = ctx.link().clone();
            move || {
                let link = link.clone();
                async move {
                    link.send_message(msg());
                    Ok::<(), anyhow::Error>(())
                }
            }
        };

        let km = generate_keymap!(
            ("C-A-m", send(|| AppMsg::UseBackend(Backend::Mdx))),
            ("C-A-e", send(|| AppMsg::UseBackend(Backend::ECDict))),
            ("C-<Return>", send(|| AppMsg::QueryDictFromInput)),
        )
        .unwrap();

        self.keybinding.push_keymap("dict", km);

        self.keybinding.setup_on_keydown(|f| {
            let root = self.input_node.cast::<HtmlInputElement>().unwrap();
            root.set_onkeydown(f);
        });
    }
}

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.schedule().add_once_task(WebStage::Init, init);
        Ok(())
    }
}

fn render(params: &RouteParams) -> Html {
    html! {
        <App query={ params.get("query").cloned().unwrap_or_default() }/>
    }
}

async fn init(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/dict/:query", render);
    router.add("/dict/", render);
    Ok(())
}
