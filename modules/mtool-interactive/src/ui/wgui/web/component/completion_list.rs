use gloo_utils::document;
use mtool_wgui::{app::AppContext, generate_keymap, KeyMap, Keybinding, SharedAction};
use serde::{Deserialize, Serialize};
use tracing::warn;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, ScrollIntoViewOptions, ScrollLogicalPosition};
use yew::{platform::spawn_local, prelude::*};

use crate::ui::wgui::model::{CompletionExit, CompletionItem};

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: Classes,
    pub id: String,
    pub input: String,
    pub keybinding: Keybinding,
}

pub struct CompletionList {
    items: Vec<CompletionItem>,
    focused_item_index: usize,
    keymap: KeyMap<SharedAction>,

    context: AppContext,
    _context_listener: ContextHandle<AppContext>,
}

#[derive(Clone)]
pub enum Msg {
    FetchCompleteRead(Vec<CompletionItem>),
    Next,
    Prev,
    FocusChanged(usize),
    Exit,
    ContextUpdated(AppContext),
}

impl Component for CompletionList {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _context_listener) = ctx
            .link()
            .context(ctx.link().callback(Msg::ContextUpdated))
            .expect("No Message Context Provided");

        let self_ = Self {
            items: Vec::new(),
            focused_item_index: 0,
            keymap: Self::generate_keymap(ctx),

            context,
            _context_listener,
        };

        Self::fetch_complete(ctx);

        self_
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchCompleteRead(items) => {
                self.items = items;
                self.set_keymap(ctx);
                true
            }
            Msg::Next => {
                let index = if self.focused_item_index == self.items.len() - 1 {
                    0
                } else {
                    self.focused_item_index + 1
                };

                self.focused_item_index = index;

                true
            }
            Msg::Prev => {
                let index = if self.focused_item_index == 0 {
                    self.items.len() - 1
                } else {
                    self.focused_item_index - 1
                };

                self.focused_item_index = index;

                true
            }
            Msg::Exit => {
                self.complete_exit();
                false
            }
            Msg::FocusChanged(index) => {
                self.focused_item_index = index;
                true
            }
            Msg::ContextUpdated(context) => {
                self.context = context;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let focus_class = |i| {
            if self.focused_item_index == i {
                "bg-gray-800"
            } else {
                ""
            }
        };

        let mut cont_class = ctx.props().class.clone();
        cont_class.extend(classes!("max-h-[17.5rem]"));

        html! {
            <>
                if !self.items.is_empty() {
                    <div
                        class={cont_class}
                        tabindex=0>
                    {
                        for self.items.iter().enumerate().map(|(i, item)| {
                            html! {
                                <div
                                    id={ Self::completion_item_id(i) }
                                    class={classes!("flex",
                                                "h-14",
                                                "items-center",
                                                "px-4",
                                                focus_class(i))}
                                    onclick={ ctx.link().callback(move |_| Msg::FocusChanged(i)) }>
                                    <div class={classes!("font-mono",
                                                         "text-2xl")}>
                                      { self.render_template(&item) }
                                    </div>
                                </div>
                            }
                        })
                    }
                    </div>
                }
            </>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.focused_item_index = 0;

        Self::fetch_complete(ctx);

        true
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.scroll_into_focused_item();
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        ctx.props()
            .keybinding
            .remove_keymap("COMPLETION_LIST_KEYMAP");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionExitArgs {
    pub v: CompletionExit,
}

impl CompletionList {
    const COMPLETION_LIST_KEYMAP: &str = "completion_list";

    fn fetch_complete(ctx: &Context<Self>) {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct CompletionArgs {
            pub completed: String,
        }

        let input = ctx.props().input.to_string();
        ctx.link().send_future(async move {
            Msg::FetchCompleteRead(
                match mtauri_sys::invoke(
                    "plugin:interactive::completion|complete",
                    &CompletionArgs { completed: input },
                )
                .await
                {
                    Ok(items) => items,
                    Err(e) => {
                        warn!("invoke complete failed: {:?}", e);
                        vec![]
                    }
                },
            )
        });
    }

    fn generate_keymap(ctx: &Context<Self>) -> KeyMap<SharedAction> {
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
        generate_keymap!(
            ("C-n", send(Msg::Next)),
            ("C-p", send(Msg::Prev)),
            ("<Return>", send(Msg::Exit)),
        )
        .unwrap()
    }

    fn scroll_into_focused_item(&self) {
        if let Some(elm) = self.focused_item() {
            let mut opt = ScrollIntoViewOptions::new();
            opt.block(ScrollLogicalPosition::Nearest);

            elm.scroll_into_view_with_scroll_into_view_options(&opt);
        }
    }

    fn set_keymap(&self, ctx: &Context<Self>) {
        let items = &self.items;
        if items.is_empty() {
            ctx.props()
                .keybinding
                .remove_keymap(Self::COMPLETION_LIST_KEYMAP);
        } else {
            ctx.props()
                .keybinding
                .push_keymap(Self::COMPLETION_LIST_KEYMAP, self.keymap.clone());
        }
    }

    fn completion_item_id(i: usize) -> String {
        format!("completion-item-{}", i)
    }

    fn focused_item(&self) -> Option<HtmlElement> {
        document()
            .get_element_by_id(&Self::completion_item_id(self.focused_item_index))
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    }

    fn complete_exit(&self) {
        let item = self.items[self.focused_item_index].to_owned();
        spawn_local(async move {
            if let Err(e) = mtauri_sys::invoke::<CompletionExitArgs, ()>(
                "plugin:interactive::completion|complete_exit",
                &CompletionExitArgs {
                    v: CompletionExit::Id(item.id),
                },
            )
            .await
            {
                warn!("invoke complete_exit failed: {:?}", e)
            }
        });
    }

    fn render_template(&self, item: &CompletionItem) -> Html {
        match self.context.templator.render(&item.template_id, &item.data) {
            Ok(view) => view,
            Err(e) => html! {
                { format!("{:?}", e) }
            },
        }
    }
}
