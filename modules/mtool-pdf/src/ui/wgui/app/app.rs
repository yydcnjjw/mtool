use anyhow::Context as _;
use mapp::provider::Res;
#[allow(unused)]
use pdfium_render::prelude::*;
use tracing::warn;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};
use yew::prelude::*;

use super::pdf::{Pdf, PdfDoc};

pub struct App {
    canvas_node: NodeRef,
    doc: Option<PdfDoc>,
}

pub enum AppMsg {
    #[allow(unused)]
    LoadPdf(PdfDoc),
    Error(anyhow::Error),
}

#[derive(Properties)]
pub struct AppProps {
    pub pdf: Res<Pdf>,
}

impl PartialEq for AppProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Component for App {
    type Message = AppMsg;

    type Properties = AppProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_node: NodeRef::default(),
            doc: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::LoadPdf(doc) => {
                self.render(&doc).unwrap();
                self.doc = Some(doc);
                true
            }
            AppMsg::Error(e) => {
                warn!("{:?}", e);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let pdf = ctx.props().pdf.clone();
        let onchange = ctx
            .link()
            .callback_future(move |e| Self::load_from_file(pdf.clone(), e));
        html! {
            <>
                <input type="file" accept=".pdf" {onchange}/>
                <canvas ref={ self.canvas_node.clone() }/>
            </>
        }
    }
}

impl App {
    #[allow(unused)]
    async fn load_from_file(pdf: Res<Pdf>, e: Event) -> AppMsg {
        match {
            |e: Event| async move {
                let e = e.target_unchecked_into::<HtmlInputElement>();
                let files = e.files().context("read files")?;
                let file = files.get(0).context("get first file")?;

                #[cfg(target_family = "wasm")]
                return Ok(AppMsg::LoadPdf(
                    pdf.load_from_blob(
                        file.slice().unwrap(),
                        // .map_err(|e| anyhow::anyhow!("convert file slice: {:?}", e))?,
                        None,
                    )
                    .await?,
                ));

                unreachable!()
            }
        }(e)
        .await
        {
            Ok(msg) => msg,
            Err(e) => AppMsg::Error(e),
        }
    }

    #[allow(unused)]
    fn render(&self, doc: &PdfDoc) -> Result<(), anyhow::Error> {
        let canvas = self
            .canvas_node
            .cast::<HtmlCanvasElement>()
            .context("cast to HtmlCanvasElement")?;

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let mut height = 0;
        for page in doc.pages().iter() {
            #[cfg(target_family = "wasm")]
            {
                let image = page
                    .render_with_config(
                        &PdfRenderConfig::new()
                            .render_form_data(true)
                            .set_target_size(720, 1080)
                            .highlight_text_form_fields(PdfColor::YELLOW.with_alpha(128))
                            .highlight_checkbox_form_fields(PdfColor::BLUE.with_alpha(128)),
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?
                    .as_image_data()
                    .unwrap();

                ctx.put_image_data(&image, 0 as f64, height as f64).unwrap();

                height += image.height();
            }
        }

        Ok(())
    }
}
