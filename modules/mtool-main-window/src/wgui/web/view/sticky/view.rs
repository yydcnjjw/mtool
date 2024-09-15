use mapp::{
    dpi::PhysicalSize,
    tracing::{debug, warn},
    wasm_bindgen::JsValue,
};
use mtool_wgui::mtauri_sys::prelude::{Event as TauriEvent, *};
use web_sys;
use yew::{platform::spawn_local, prelude::*};

use crate::wgui::generic::view::sticky;

#[derive(Properties, PartialEq)]
pub struct Props {}

pub struct View {
    command_unlisten: Option<CommandListener>,
    view_stack: Vec<sticky::TemplateView>,

    hide_window_size: Option<PhysicalSize<u32>>,

    is_dragging: bool,
    is_hide: bool,
}

pub enum Msg {
    RegisterCommandListener(CommandListener),
    ExecCommand(sticky::Command),
    LeftMouseUp,
    MouseDown(MouseEvent),
    Hide,
    Show,
    UpdateView(bool),
}

impl View {
    fn listen_command(ctx: &Context<Self>) {
        let link = ctx.link().clone();
        ctx.link().send_future(async move {
            let unlisten = match Window::current()
                .unwrap()
                .listen("sticky:command", move |e: TauriEvent<sticky::Command>| {
                    link.send_message(Msg::ExecCommand(e.payload));
                    Ok(())
                })
                .await
            {
                Ok(v) => Some(Box::new(v) as Box<dyn Fn() -> Result<(), JsValue>>),
                Err(e) => {
                    warn!("listen sticky:command event failed: {:?}", e);
                    None
                }
            };

            Msg::RegisterCommandListener(CommandListener { unlisten })
        });
    }
}

impl Component for View {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self::listen_command(ctx);
        Self {
            command_unlisten: None,
            view_stack: Default::default(),

            hide_window_size: None,
            is_dragging: false,
            is_hide: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RegisterCommandListener(unlisten) => {
                self.command_unlisten = Some(unlisten);
                false
            }
            Msg::ExecCommand(cmd) => match cmd {
                sticky::Command::ShowMain(view) => {
                    if let Some(it) = self.view_stack.iter_mut().find(|it| it.id == view.id) {
                        *it = view;
                    } else {
                        self.view_stack.push(view);
                    }
                    true
                }
                sticky::Command::LeftMouseUp => {
                    ctx.link().send_message(Msg::LeftMouseUp);
                    false
                }
            },
            Msg::Hide => {
                self.is_hide = true;
                let win = web_sys::window().unwrap();
                let width = win.outer_width().unwrap().as_f64().unwrap();
                let height = win.outer_height().unwrap().as_f64().unwrap();

                self.hide_window_size = Some(PhysicalSize::new(width as u32, height as u32));

                ctx.link().send_future(async move {
                    let win = Window::current().unwrap();

                    win.set_size(PhysicalSize::new(width as u32, 5).into())
                        .await
                        .unwrap();

                    debug!("{:?}", PhysicalSize::new(width as u32, 5));
                    Msg::UpdateView(true)
                });

                false
            }
            Msg::Show => {
                self.is_hide = false;
                let size = self.hide_window_size;

                ctx.link().send_future(async move {
                    if let Some(size) = size {
                        debug!("show {:?}", size);
                        Window::current()
                            .unwrap()
                            .set_size(size.into())
                            .await
                            .unwrap();
                    }
                    Msg::UpdateView(true)
                });

                false
            }
            Msg::LeftMouseUp => {
                debug!("mouse up");
                if self.is_dragging {
                    debug!("dragging");
                    self.is_dragging = false;
                    ctx.link().send_future(async move {
                        let win = Window::current().unwrap();
                        let pos = win.outer_position().await.unwrap();
                        debug!("{:?}", pos);
                        if pos.y <= 0 {
                            Msg::Hide
                        } else {
                            Msg::UpdateView(false)
                        }
                    });
                }
                false
            }
            Msg::MouseDown(e) => {
                // left button
                if e.ctrl_key() && e.buttons() == 1 {
                    self.is_dragging = true;
                    spawn_local(async move {
                        let win = Window::current().unwrap();
                        win.start_dragging().await.unwrap();
                    });
                }
                false
            }
            Msg::UpdateView(v) => v,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // let onmouseenter = ctx.link().callback(|_| Msg::Show);
        // let onmousedown = ctx.link().callback(move |e: MouseEvent| Msg::MouseDown(e));

        html! {}
    }
}

pub struct CommandListener {
    pub unlisten: Option<Box<dyn Fn() -> Result<(), JsValue>>>,
}

impl Drop for CommandListener {
    fn drop(&mut self) {
        if let Some(unlisten) = &self.unlisten {
            (*unlisten)().unwrap();
        }
    }
}

// use yew::prelude::*;

// #[function_component]
// pub fn view() -> yew::Html {

// }
