use std::{
    io::Cursor,
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use image::ImageFormat;
use mapp::prelude::*;
use pdfium_render::{rect::PdfRect, render_config::PdfRenderConfig};
use tauri::{
    command,
    http::{self, status::StatusCode, Uri},
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State, Wry,
};

use crate::{
    pdf::{Pdf, PdfDoc},
    ui::wgui::{Bounds, PageInfo, PdfFile, PdfInfo, PdfRenderArgs},
};

struct PdfDocument {
    doc: PdfDoc,
    info: PdfInfo,
}

impl Deref for PdfDocument {
    type Target = PdfDoc;

    fn deref(&self) -> &Self::Target {
        &self.doc
    }
}

struct PdfRender {
    pdf: Res<Pdf>,
    doc: Mutex<Option<Arc<PdfDocument>>>,
}

impl PdfRender {
    fn new(pdf: Res<Pdf>) -> Self {
        Self {
            pdf,
            doc: Mutex::new(None),
        }
    }

    fn get_doc(&self) -> Result<Arc<PdfDocument>, anyhow::Error> {
        Ok(self
            .doc
            .lock()
            .unwrap()
            .clone()
            .context("Pdf is not loaded")?)
    }

    fn load(&self, file: &PdfFile) -> Result<PdfInfo, anyhow::Error> {
        let doc = self
            .pdf
            .load(&file.path, file.password.as_ref().map(|p| p.as_str()))?;
        let info = Self::get_doc_info(&doc)?;
        *self.doc.lock().unwrap() = Some(Arc::new(PdfDocument {
            doc,
            info: info.clone(),
        }));
        Ok(info)
    }

    fn get_doc_info(doc: &PdfDoc) -> Result<PdfInfo, anyhow::Error> {
        doc.with_doc(|doc| {
            Ok(PdfInfo {
                pages: doc
                    .pages()
                    .iter()
                    .map(|page| -> Result<PageInfo, anyhow::Error> {
                        Ok(PageInfo {
                            width: (page.width().to_inches() * 96.) as i32,
                            height: (page.height().to_inches() * 96.) as i32,
                            text_segs: page
                                .text()?
                                .segments()
                                .iter()
                                .map(|seg| {
                                    let PdfRect {
                                        bottom,
                                        left,
                                        top,
                                        right,
                                    } = seg.bounds();
                                    Bounds {
                                        bottom: (bottom.to_inches() * 96.) as isize,
                                        left: (left.to_inches() * 96.) as isize,
                                        top: (top.to_inches() * 96.) as isize,
                                        right: (right.to_inches() * 96.) as isize,
                                    }
                                })
                                .collect(),
                        })
                    })
                    .try_collect()?,
            })
        })
    }
}

#[command]
fn load_pdf(render: State<'_, PdfRender>, file: PdfFile) -> Result<PdfInfo, serde_error::Error> {
    render.load(&file).map_err(|e| serde_error::Error::new(&*e))
}

pub fn pdf_protocol_handler<R: Runtime>(
    handle: &AppHandle<R>,
    req: &http::Request,
) -> Result<http::Response, anyhow::Error> {
    let render: State<PdfRender> = handle.try_state().context("get State<PdfRender>")?;

    let args = PdfRenderArgs::decode(&Uri::from_str(req.uri())?.path()[1..])?;

    let doc = render.get_doc()?;
    let page = doc.page(args.page_index)?;

    let page_info = doc.info.pages.get(args.page_index as usize).context("")?;

    let mut buf = Cursor::new(Vec::new());
    page.render_with_config(
        &PdfRenderConfig::new().set_target_size(page_info.width, page_info.height),
    )?
    .as_image()
    .as_rgba8()
    .context("convert to image")?
    .write_to(&mut buf, ImageFormat::Png)?;

    http::ResponseBuilder::new()
        .status(StatusCode::OK)
        .mimetype("image/png")
        .body(buf.into_inner())
        .map_err(|e| anyhow::anyhow!("Building http response error: {}", e))
}

pub fn init(pdf: Res<Pdf>) -> TauriPlugin<Wry> {
    Builder::new("pdfrender")
        .setup(move |app, _| {
            app.manage(PdfRender::new(pdf));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_pdf])
        .build()
}
