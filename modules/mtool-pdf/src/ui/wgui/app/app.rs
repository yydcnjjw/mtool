use mtauri_sys::window::{Size, Window};
use mtool_wgui::component::error::render_result_view;
use serde::Serialize;
use tracing::warn;
use wasm_bindgen::prelude::*;
use web_sys::HtmlDivElement;
use yew::{platform::spawn_local, prelude::*};

use crate::ui::wgui::{PageInfo, PdfFile, PdfDocumentInfo, ScaleEvent, ScrollEvent, WPdfEvent};

pub struct App {
    root: NodeRef,

    pdf_info: Option<Result<PdfDocumentInfo, anyhow::Error>>,

    scale: f32,
}

pub enum DeviceEvent {
    Wheel(WheelEvent),
    Scroll(ScrollEvent),
}

pub enum AppMsg {
    PdfLoaded(Result<PdfDocumentInfo, anyhow::Error>),
    DeviceEvent(DeviceEvent),
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
            scale: 1.,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        Self::send_load_pdf(ctx);
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::PdfLoaded(info) => {
                if let Ok(info) = info.as_ref() {
                    Self::adjust_window(info.clone());
                }

                self.pdf_info = Some(info);
                true
            }
            AppMsg::DeviceEvent(e) => {
                if let Err(e) = self.handle_device_event(e) {
                    warn!("{:?}", e);
                }
                true
            }
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
                                "flex",
                                "flex-row",
                                "justify-center",
                                "items-start",
                                "overflow-y-scroll",
                                "bg-transparent",
                                "relative")}
                ref={self.root.clone()}
                {onwheel}
                {onscroll}
              >
                {
                    if let Some(pdf_info) = self.pdf_info.as_ref() {
                      match pdf_info {
                        Ok(info) => render_result_view(self.render_pdf(ctx, info)),
                        Err(e) => html! { <div>{ format!("{:?}", e) }</div> }
                      }
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

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
}

impl App {
    fn send_load_pdf(ctx: &Context<Self>) {
        let path = ctx.props().path.clone();
        ctx.link()
            .send_future(async move { AppMsg::PdfLoaded(Self::load_pdf(&path).await) });
    }

    async fn load_pdf(path: &str) -> Result<PdfDocumentInfo, anyhow::Error> {
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

    fn render_pdf(&self, _ctx: &Context<Self>, info: &PdfDocumentInfo) -> Result<Html, anyhow::Error> {
        if info.pages.is_empty() {
            anyhow::bail!("pdf page is empty")
        }

        let (doc_width, doc_height) = (info.width(), info.height());

        let scale = self.scale;
        let (width, height) = (doc_width as f32 * scale, doc_height as f32 * scale);

        Ok(html! {
            <>
              <div
                class={classes!("flex",
                                "flex-col",
                                "items-center")}
                style={format!(r#"
width: {width}px;
height: {height}px;
"#)}
                // {onmousedown}
              >
                {
                    for info.pages.iter().enumerate()
                        .map(|(i, info)| html! {
                            <PdfPage
                                index={i as i32}
                                info={info.clone()}
                                scale={scale}/>
                        })
                }
              </div>
            </>
        })
    }

    fn adjust_window(info: PdfDocumentInfo) {
        async fn adjust_window_inner(info: &PdfDocumentInfo) -> Result<(), JsValue> {
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
                    self.send_pdf_event(WPdfEvent::Scale(ScaleEvent {
                        scale: self.scale,
                        mouse_point: (e.client_x(), e.client_y()),
                    }))
                }
            }
            DeviceEvent::Scroll(e) => {
                self.send_pdf_event(WPdfEvent::Scroll(e));
            }
        }
        Ok(())
    }

    fn send_pdf_event(&self, e: WPdfEvent) {
        let window = Window::current().unwrap();
        spawn_local(async move {
            if let Err(e) = window.emit("pdf-event", &e).await {
                warn!("{:?}", e);
            }
        });
    }
}

#[derive(Properties, Clone)]
struct PdfPageProps {
    index: i32,

    info: PageInfo,

    scale: f32,
}

impl PartialEq for PdfPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.scale == other.scale
    }
}

#[function_component]
fn PdfPage(props: &PdfPageProps) -> Html {
    fn render(props: &PdfPageProps) -> Result<Html, anyhow::Error> {
        let PdfPageProps {
            index: _,
            info,
            scale,
        } = props;

        Ok(html! {
            <div style={
                format!(r#"
width: {}px;
height: {}px;
"#, info.width as f32 * scale, info.height as f32 * scale)
            }>
            </div>
        })
    }
    render_result_view(render(props))
}
