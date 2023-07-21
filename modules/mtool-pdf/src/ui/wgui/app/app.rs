use anyhow::Context as _;
use gloo_utils::document;
use mtauri_sys::window::{Size, Window};
use mtool_wgui::component::error::render_result_view;
use serde::Serialize;
use tracing::{debug, warn};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{platform::spawn_local, prelude::*};

use crate::ui::wgui::{PageInfo, PdfFile, PdfInfo, PdfRenderArgs};

pub struct App {
    pdf_info: Option<Result<PdfInfo, anyhow::Error>>,
}

pub enum AppMsg {
    Pdf(Result<PdfInfo, anyhow::Error>),
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
        Self { pdf_info: None }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        Self::send_load_pdf(ctx);
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Pdf(info) => {
                if let Ok(info) = info.as_ref() {
                    let info = info.clone();
                    spawn_local(async move {
                        if let Err(e) = Self::adjust_window(&info).await {
                            warn!("adjust window error: {:?}", e);
                        }
                    });
                }

                self.pdf_info = Some(info);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
              <div class={classes!("w-screen",
                                   "h-screen",
                                   "overflow-scroll")}>
                {
                    if let Some(pdf_info) = self.pdf_info.as_ref() {
                      match pdf_info {
                        Ok(info) => render_result_view(self.render_pdf(info)),
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
}

impl App {
    fn send_load_pdf(ctx: &Context<Self>) {
        let path = ctx.props().path.clone();
        ctx.link()
            .send_future(async move { AppMsg::Pdf(Self::load_pdf(&path).await) });
    }

    async fn load_pdf(path: &str) -> Result<PdfInfo, anyhow::Error> {
        #[derive(Serialize)]
        struct Args {
            file: PdfFile,
        }
        mtauri_sys::invoke(
            "plugin:pdfrender|load_pdf",
            &Args {
                file: PdfFile {
                    path: path.to_string(),
                    password: None,
                },
            },
        )
        .await
    }

    fn render_pdf(&self, info: &PdfInfo) -> Result<Html, anyhow::Error> {
        if info.pages.is_empty() {
            anyhow::bail!("pdf page is empty")
        }

        let pdf_width = info.pages[0].width;
        let pdf_height: i32 = info.pages.iter().map(|page| page.height).sum();

        Ok(html! {
            <div
              class={classes!("flex",
                              "flex-col",
                              "items-center")}
              style={format!(r#"
width: {}px;
height: {}px;
"#, pdf_width, pdf_height)}>
              {
                for info.pages.iter().enumerate().map(|(i, info)| html! { <PdfPage index={i as i32} info={info.clone()}/> })
              }
            </div>
        })
    }

    async fn adjust_window(info: &PdfInfo) -> Result<(), JsValue> {
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
}

fn convert_file_src(path: &str, protocol: &str) -> Result<String, JsValue> {
    Ok(
        if window()
            .unwrap()
            .navigator()
            .user_agent()?
            .contains("Windows")
        {
            format!("https://{protocol}.localhost/{path}")
        } else {
            format!("{protocol}://localhost/{path}")
        },
    )
}

fn generate_page_url(i: i32) -> Result<String, anyhow::Error> {
    Ok(format!(
        "{}",
        convert_file_src(
            &PdfRenderArgs {
                page_index: i as u16
            }
            .encode()?,
            "pdfviewer"
        )
        .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    ))
}

#[derive(Properties, Clone)]
struct PdfPageProps {
    index: i32,
    info: PageInfo,
}

impl PartialEq for PdfPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[function_component]
fn PdfPage(props: &PdfPageProps) -> Html {
    let canvas = use_memo(
        |props| {
            fn render(props: &PdfPageProps) -> Result<Html, JsValue> {
                let canvas = document().create_element("canvas")?;
                let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

                canvas.set_width(props.info.width as u32);
                canvas.set_height(props.info.height as u32);

                let context = canvas
                    .get_context("2d")?
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()?;

                context.translate(0., canvas.height() as f64)?;
                context.scale(1., -1.)?;

                context.begin_path();

                for bounds in props.info.text_segs.iter() {
                    let x = bounds.left as f64;
                    let y = bounds.bottom as f64;
                    let width = (bounds.right - bounds.left) as f64;
                    let height = (bounds.top - bounds.bottom) as f64;
                    context.rect(x, y, width, height)
                }

                context.stroke();

                Ok(Html::VRef(canvas.into()))
            }
            render_result_view(render(props).map_err(|e| anyhow::anyhow!("{:?}", e)))
        },
        props.clone(),
    );

    fn render(props: &PdfPageProps, canvas: Html) -> Result<Html, anyhow::Error> {
        let PdfPageProps { index, info } = props;

        Ok(html! {
            <div style={
                format!(r#"
width: {}px;
height: {}px;
background-image: url('{}');
"#, info.width, info.height, generate_page_url(*index)?)
            }>
                { canvas }
            </div>
        })
    }
    render_result_view(render(props, (*canvas).clone()))
}
