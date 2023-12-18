use std::rc::Rc;

use mtauri_sys::window::{Size, Window};
use mtool_wgui::{component::error::render_result_view, generate_keymap, Keybinding};
use serde::Serialize;
use tracing::warn;
use wasm_bindgen::prelude::*;
use web_sys::HtmlDivElement;
use yew::{platform::spawn_local, prelude::*};

use super::event::{
    self, PageInfo, PdfDocumentInfo, PdfFile, ScaleEvent, ScrollEvent, WMouseEvent, WPdfEvent,
    WPdfLoadEvent,
};

pub struct App {
    root: NodeRef,

    pdf_info: Option<Rc<event::PdfDocumentInfo>>,

    keybinding: Keybinding,

    scale: f32,

    pdf_load_unlisten: Option<PdfLoadListener>,
}

pub enum DeviceEvent {
    Wheel(WheelEvent),
    Scroll(ScrollEvent),
}

pub enum AppMsg {
    PdfLoadEvent(WPdfLoadEvent),
    RegisterPdfLoadListener(PdfLoadListener),

    DeviceEvent(DeviceEvent),

    Error(anyhow::Error),
    None,
}

#[derive(Properties, PartialEq)]
pub struct AppProps {
    pub path: String,
}

impl Component for App {
    type Message = AppMsg;

    type Properties = AppProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::send_load_pdf(ctx);

        Self {
            root: NodeRef::default(),
            pdf_info: None,

            keybinding: Keybinding::new(),

            scale: 1.,

            pdf_load_unlisten: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        Self::send_load_pdf(ctx);
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::PdfLoadEvent(e) => match e {
                WPdfLoadEvent::DocLoaded(info) => {
                    let info = Rc::new(info);
                    Self::adjust_window(info.clone());
                    self.pdf_info = Some(info);
                    true
                }
                _ => false,
            },
            AppMsg::RegisterPdfLoadListener(unlisten) => {
                self.pdf_load_unlisten = Some(unlisten);
                false
            }
            AppMsg::DeviceEvent(e) => {
                if let Err(e) = self.handle_device_event(e) {
                    warn!("{:?}", e);
                }
                true
            }
            AppMsg::Error(e) => {
                warn!("{:?}", e);
                false
            }
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onwheel = ctx
            .link()
            .callback(|e| AppMsg::DeviceEvent(DeviceEvent::Wheel(e)));

        let onscroll = {
            let root = self.root.clone();
            ctx.link().callback(move |_| {
                let root = root.cast::<HtmlDivElement>().unwrap();
                AppMsg::DeviceEvent(DeviceEvent::Scroll(ScrollEvent {
                    left: root.scroll_left(),
                    top: root.scroll_top(),
                }))
            })
        };

        html! {
            <>
              <div
                class={classes!("w-screen",
                                "h-screen",
                                "overflow-scroll",
                                "bg-transparent",
                                "relative")}
                ref={self.root.clone()}
                {onwheel}
                {onscroll}
              >
                {
                  if let Some(pdf_info) = self.pdf_info.as_ref() {
                    render_result_view(self.render_pdf(ctx, &pdf_info))
                  } else {
                    html! {
                      <div>{"Loading pdf"}</div>
                    }
                  }
                }
              </div>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.register_keybinding(ctx)
        }
    }
}

impl App {
    fn send_load_pdf(ctx: &Context<Self>) {
        let link = ctx.link().clone();
        ctx.link().send_future(async move {
            let unlisten = match Window::current()
                .unwrap()
                .listen(
                    "pdf_load",
                    move |e: mtauri_sys::event::Event<WPdfLoadEvent>| {
                        link.send_message(AppMsg::PdfLoadEvent(e.payload));
                        Ok(())
                    },
                )
                .await
            {
                Ok(v) => Some(Box::new(v) as Box<dyn Fn() -> Result<(), JsValue>>),
                Err(e) => {
                    warn!("listen route event failed: {:?}", e);
                    None
                }
            };
            AppMsg::RegisterPdfLoadListener(PdfLoadListener { unlisten })
        });

        let path = ctx.props().path.clone();
        ctx.link().send_future(async move {
            if let Err(e) = Self::load_pdf(&path).await {
                AppMsg::Error(e)
            } else {
                AppMsg::None
            }
        });
    }

    async fn load_pdf(path: &str) -> Result<(), anyhow::Error> {
        #[derive(Serialize)]
        struct Args {
            file: PdfFile,
        }
        mtauri_sys::invoke(
            "plugin:pdfloader|load_pdf",
            &Args {
                file: PdfFile {
                    path: path.to_string(),
                    password: None,
                },
            },
        )
        .await
    }

    fn render_pdf(
        &self,
        _ctx: &Context<Self>,
        info: &PdfDocumentInfo,
    ) -> Result<Html, anyhow::Error> {
        if info.pages.is_empty() {
            anyhow::bail!("pdf page is empty")
        }

        let (doc_width, doc_height) = (info.width(), info.height());

        let scale = self.scale;
        let (width, height) = (
            (doc_width as f32 * scale).round(),
            (doc_height as f32 * scale).round(),
        );

        Ok(html! {
            <>
              <div
                class={classes!("mx-auto")}
                style={format!(r#"
width: {width}px;
height: {height}px;
"#)}
              >
                {
                    for info.pages.iter().enumerate()
                        .map(|(i, info)| html! {
                            <PdfPage
                                pageindex={i as u16}
                                info={info.clone()}
                                scale={scale}/>
                        })
                }
              </div>
            </>
        })
    }

    fn adjust_window(info: Rc<PdfDocumentInfo>) {
        async fn adjust_window_inner(info: &event::PdfDocumentInfo) -> Result<(), JsValue> {
            if let Some(page) = info.pages.get(0) {
                let window = Window::current()?;
                window
                    .set_size(Size::new_physical(
                        (page.width + 10) as usize,
                        (page.height as usize).max(960),
                    ))
                    .await?;
                window.center().await?;
            }

            Ok(())
        }

        spawn_local(async move {
            if let Err(e) = adjust_window_inner(&info).await {
                warn!("adjust window error: {:?}", e);
            }
        });
    }

    fn handle_device_event(&mut self, e: DeviceEvent) -> Result<(), anyhow::Error> {
        match e {
            DeviceEvent::Wheel(e) => {
                if e.delta_mode() != WheelEvent::DOM_DELTA_PIXEL {
                    anyhow::bail!("only support DOM_DELTA_PIXEL");
                }

                if e.ctrl_key() {
                    let delta = e.delta_y() as f32;
                    let delta = delta * if self.scale > 1. { -0.001 } else { -0.0005 };
                    self.scale = (self.scale + delta).clamp(0.25, 5.);
                    send_pdf_event(WPdfEvent::Scale(ScaleEvent {
                        scale: self.scale,
                        mouse_point: (e.client_x(), e.client_y()),
                    }))
                }
            }
            DeviceEvent::Scroll(e) => {
                send_pdf_event(WPdfEvent::Scroll(e));
            }
        }
        Ok(())
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

        let km = generate_keymap!(("p", send(|| AppMsg::None)),).unwrap();

        self.keybinding.push_keymap("pdfviewer", km);

        self.keybinding.setup_on_keydown(|f| {
            let root = self.root.cast::<HtmlDivElement>().unwrap();
            root.set_onkeydown(f);
        });
    }
}

#[derive(Properties, Clone)]
struct PdfPageProps {
    pageindex: u16,

    info: PageInfo,

    scale: f32,
}

impl PartialEq for PdfPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.pageindex == other.pageindex && self.scale == other.scale
    }
}

#[function_component]
fn PdfPage(props: &PdfPageProps) -> Html {
    fn render(props: &PdfPageProps) -> Result<Html, anyhow::Error> {
        let PdfPageProps {
            pageindex,
            info,
            scale,
        } = props;

        let (onmousemove, onmouseup, onmousedown) = {
            let page_index = *pageindex;
            (
                Callback::from(move |e: MouseEvent| {
                    send_pdf_event(WPdfEvent::Mouse {
                        page_index,
                        e: event::MouseEvent::Move(WMouseEvent::from(e)),
                    });
                }),
                Callback::from(move |e: MouseEvent| {
                    send_pdf_event(WPdfEvent::Mouse {
                        page_index,
                        e: event::MouseEvent::Up(WMouseEvent::from(e)),
                    });
                }),
                Callback::from(move |e: MouseEvent| {
                    send_pdf_event(WPdfEvent::Mouse {
                        page_index,
                        e: event::MouseEvent::Down(WMouseEvent::from(e)),
                    });
                }),
            )
        };

        Ok(html! {
            <div
              style={
                    format!(r#"
width: {}px;
height: {}px;
"#, (info.width as f32 * scale).round(), (info.height as f32 * scale).round())}
              {onmousemove}
              {onmouseup}
              {onmousedown}
            >
            </div>
        })
    }
    render_result_view(render(props))
}

fn send_pdf_event(e: WPdfEvent) {
    let window = Window::current().unwrap();
    spawn_local(async move {
        if let Err(e) = window.emit("pdf-event", &e).await {
            warn!("{:?}", e);
        }
    });
}

pub struct PdfLoadListener {
    pub unlisten: Option<Box<dyn Fn() -> Result<(), JsValue>>>,
}

impl Drop for PdfLoadListener {
    fn drop(&mut self) {
        if let Some(unlisten) = &self.unlisten {
            (*unlisten)().unwrap();
        }
    }
}
