use std::collections::HashMap;

use mtauri_sys::window::{Window, PhysicalSize};
use mtool_wgui::prelude::*;
use tracing::warn;
use wasm_bindgen::JsValue;
use yew::prelude::*;

use crate::wgui::generic::view::sticky;

#[derive(Properties, PartialEq)]
pub struct Props {}

pub struct View {
    command_unlisten: Option<CommandListener>,
    subview: HashMap<String, sticky::TemplateView>,
}

pub enum Msg {
    RegisterCommandListener(CommandListener),
    ExecCommand(sticky::Command),
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
            subview: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RegisterCommandListener(unlisten) => {
                self.command_unlisten = Some(unlisten);
                false
            }
            Msg::ExecCommand(cmd) => match cmd {
                sticky::Command::ShowSubview(view) => {
                    self.subview.insert(view.id.clone(), view);
                    true
                }
            },
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <AutoWindow window={
                WindowProps{
                    horizontal: Horizontal::RightAlign(12),
                    vertical: Vertical::Absolute(24),
                    initial_size: PhysicalSize::new(350, 350),
                    resizable: true,
                    ..Default::default()
                }
            }>
              <div class={classes!(
                  "w-screen",
                  "h-screen",
                  "flex",
                  "flex-col",
                  "items-center",
                  "justify-center",
                )}>
                <div class={classes!(
                    "flex",
                    "flex-col",
                    "p-1",
                    "w-full",
                    "h-full",
                    "rounded-md",
                    "shadow-md",
                    "text-white",
                    "bg-gray-600/75",
                  )}>
                  {
                    self.subview.iter().map(|(_, view)| html! {
                      <div class={classes!("")}>
                        <TemplateView
                          template_id={ view.template_id.clone() }
                          data={ view.template_data.clone() }/>
                      </div>
                    }).collect::<Html>()
                  }
                </div>
              </div>
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
