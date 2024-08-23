use gloo_events::EventListener;
use mtauri_sys::window::{PhysicalPosition, PhysicalSize, Window};
use mtool_wgui::prelude::*;
use tracing::{debug, warn};
use wasm_bindgen::{JsCast, JsValue};
use yew::{platform::spawn_local, prelude::*};

use crate::wgui::generic::view::sticky;

#[derive(Properties, PartialEq)]
pub struct Props {}

pub struct View {
    command_unlisten: Option<CommandListener>,
    main_view_lst: Vec<sticky::TemplateView>,
    sub_view_lst: Vec<sticky::TemplateView>,

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
    NeedRender(bool),
}

impl View {
    fn listen_command(ctx: &Context<Self>) {
        let link = ctx.link().clone();
        ctx.link().send_future(async move {
            let unlisten = match Window::current()
                .unwrap()
                .listen(
                    "sticky:command",
                    move |e: mtauri_sys::event::Event<sticky::Command>| {
                        link.send_message(Msg::ExecCommand(e.payload));
                        Ok(())
                    },
                )
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
            main_view_lst: Default::default(),
            sub_view_lst: Default::default(),

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
                    if let Some(it) = self.main_view_lst.iter_mut().find(|it| it.id == view.id) {
                        *it = view;
                    } else {
                        self.main_view_lst.push(view);
                    }
                    true
                }
                sticky::Command::ShowSub(view) => {
                    if let Some(it) = self.sub_view_lst.iter_mut().find(|it| it.id == view.id) {
                        *it = view;
                    } else {
                        self.sub_view_lst.push(view);
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
                    Msg::NeedRender(true)
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
                    Msg::NeedRender(true)
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
                            Msg::NeedRender(false)
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
            Msg::NeedRender(v) => v,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        fn notify_style(n: usize) -> String {
            (0..n)
                .map(|i| {
                    format!(
                        r#"
@container notify (min-height: {1}rem) {{
    .\@h-\[{1}rem\]\/notify\:h-\[2rem\] {{
        height: 2rem;
    }}
}}

@container notify (min-height: {0}rem) {{
    .\@h-\[{0}rem\]\/notify\:h-\[4rem\] {{
        height: 4rem;
    }}
}}

@container notify (max-height: {1}rem) {{
    .\@h-max-\[{1}rem\]\/notify\:hidden {{
        display: none;    
    }}
}}
        "#,
                        (i + 1) * 4,
                        (i + 1) * 4 - 2
                    )
                })
                .collect()
        }

        let onmouseenter = ctx.link().callback(|_| Msg::Show);

        let onmousedown = ctx.link().callback(move |e: MouseEvent| Msg::MouseDown(e));

        html! {
        <AutoWindow
          window={WindowProps{
          horizontal: Horizontal::RightAlign(12),
          vertical: Vertical::TopAlign(0),
          initial_size: PhysicalSize::new(350, 350),
          resizable: true,
          ..Default::default()
          }}
          >
          {
          if self.is_hide {
              html! {
                  <div
                  {onmouseenter}
                  class={classes!("w-screen","h-screen", "bg-black")}/>
              }
          } else {
              html! {
                            <div
            class={classes!(
            "w-screen",
            "h-screen",
            "text-white",
            )}
            {onmousedown}
            >
            <div
              class={classes!(
              "flex",
              "flex-col",
              "p-2",
              "gap-y-2",
              "w-full",
              "h-full",
              "rounded-md",
              "bg-gray-600/75",
              )}
              >
              <div
                class={classes!(
                "basis-2/3",
                "overflow-y-auto",
                )}
                >
                if let Some(view) = self.main_view_lst.last() {
                <div class={classes!("")}>
                  <TemplateView
                    template_id={view.template_id.clone()}
                    data={view.template_data.clone()}
                    />
                </div>
                }
              </div>
              <style>
                { notify_style(self.sub_view_lst.len()) }
              </style>
              <div
                class={classes!(
                "@container-size/notify",
                "basis-1/3",
                "flex",
                "flex-col",
                "justify-end",
                )}
                >
                { self.sub_view_lst.iter().enumerate().map(|(i, view)| html!{
                <div class={classes!(
                     "w-full",
                     "flex-none",
                     "bg-primary",
                     format!("@h-[{}rem]/notify:h-[4rem]", (i + 1) * 4),
                     format!("@h-[{}rem]/notify:h-[2rem]", ((i + 1) * 4) - 2),
                     format!("@h-max-[{}rem]/notify:hidden", ((i + 1) * 4) - 2),
                     )}>
                  <div class={classes!(
                       "w-full",
                       "h-full"
                       )}
                       >
                    <TemplateView
                      template_id={ view.template_id.clone() }
                      data={ view.template_data.clone() }/>
                  </div>
                </div>
                }).collect::
                <Html>() }
              </div>
            </div>
          </div>
              }
          }
          }
        </AutoWindow>
        }
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
