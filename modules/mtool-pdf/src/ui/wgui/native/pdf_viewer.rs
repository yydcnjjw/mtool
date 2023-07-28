use std::{ffi::c_void, sync::Arc};

use anyhow::Context;
use itertools::Itertools;
use mtool_wgui::WGuiWindow;
use pdfium_render::prelude::*;
use skia_safe as sk;
use tauri::{LogicalSize, Manager, PhysicalPosition, PhysicalSize, WindowEvent};
use tokio::sync::{broadcast, watch};
use tracing::{debug, warn};

use super::{pdf_document::PdfDocument, pdf_page::TextRange};
use crate::{
    pdf::Pdf,
    ui::wgui::{
        service::{PdfDocument as Document, PdfLoader},
        ScaleEvent, ScrollEvent, WPdfEvent,
    },
};

struct PdfRenderer {
    doc: Option<PdfDocument>,

    surface: sk::Surface,

    bitmap: PdfBitmap<'static>,

    image_snapshot: watch::Sender<sk::Image>,

    size: PhysicalSize<u32>,
    scale: f32,
    scroll_offset: PhysicalPosition<i32>,
}

unsafe impl Send for PdfRenderer {}

impl PdfRenderer {
    fn new(size: PhysicalSize<u32>) -> Result<(Self, watch::Receiver<sk::Image>), anyhow::Error> {
        let PhysicalSize { width, height } = size.cast::<i32>();
        let mut surface =
            sk::surfaces::raster_n32_premul((width, height)).context("create pdf surface")?;

        let (image_snapshot, rx) = watch::channel(surface.image_snapshot());

        Ok((
            Self {
                doc: None,
                surface,
                bitmap: PdfBitmap::empty(
                    width,
                    height,
                    PdfBitmapFormat::BGRA,
                    Pdf::get_unwrap().bindings(),
                )?,
                image_snapshot,

                scale: 1.,
                size,
                scroll_offset: PhysicalPosition::new(0, 0),
            },
            rx,
        ))
    }

    fn canvas(&mut self) -> &mut sk::Canvas {
        self.surface.canvas()
    }

    fn draw_selection(
        &mut self,
        page: &PdfPage,
        (page_width, page_height): (i32, i32),

        text_range: &TextRange,
    ) -> Result<(), anyhow::Error> {
        let bindings = page.bindings();
        let page_handle = page.page_handle();
        let text_page = page.text()?;
        let text_page_handle = text_page.handle();
        let count = bindings.FPDFText_CountRects(
            *text_page_handle,
            text_range.index as i32,
            text_range.count as i32,
        );

        let mut highlight_rects = Vec::new();
        let mut line_rect = sk::Rect::new_empty();
        for i in 0..count {
            let (mut page_left, mut page_top, mut page_right, mut page_bottom) = (0., 0., 0., 0.);
            if bindings.FPDFText_GetRect(
                *text_page_handle,
                i,
                &mut page_left,
                &mut page_top,
                &mut page_right,
                &mut page_bottom,
            ) == 1
            {
                let mut left = 0;
                let mut top = 0;
                let mut right = 0;
                let mut bottom = 0;
                bindings.FPDF_PageToDevice(
                    page_handle,
                    0,
                    0,
                    page_width,
                    page_height,
                    page.rotation()?.as_pdfium(),
                    page_left,
                    page_top,
                    &mut left,
                    &mut top,
                );

                bindings.FPDF_PageToDevice(
                    page_handle,
                    0,
                    0,
                    page_width,
                    page_height,
                    page.rotation()?.as_pdfium(),
                    page_right,
                    page_bottom,
                    &mut right,
                    &mut bottom,
                );

                let rect = sk::Rect::new(left as f32, top as f32, right as f32, bottom as f32);
                if line_rect.is_empty() {
                    line_rect = rect;
                } else {
                    if sk::Rect::new(f32::MIN, line_rect.top, f32::MAX, line_rect.bottom)
                        .intersect(sk::Rect::new(f32::MIN, top as f32, f32::MAX, bottom as f32))
                    {
                        line_rect.join(rect);
                    } else {
                        highlight_rects.push(line_rect);
                        line_rect = rect;
                    }
                }
            }
        }

        highlight_rects.push(line_rect);

        let mut paint = sk::Paint::new(sk::Color4f::from(sk::Color::from_rgb(153, 193, 218)), None);
        paint.set_blend_mode(sk::BlendMode::Multiply);
        for rect in highlight_rects {
            self.canvas().draw_rect(rect, &paint);
        }

        Ok(())
    }

    fn render_pdf(&mut self, doc: PdfDocument) -> Result<(), anyhow::Error> {
        let PhysicalSize { width, height } = self.size;
        let (doc_width, _doc_height) = (doc.width(), doc.height());

        let doc_viewpoint = sk::IRect::new(
            0,
            self.scroll_offset.y,
            doc_width as i32,
            self.scroll_offset.y + height as i32,
        );

        debug!("doc_viewpoint={:?}", doc_viewpoint);

        let mut doc_top_offset = 0;

        let pages = doc
            .document_info()
            .pages
            .iter()
            .enumerate()
            .filter_map(|(i, size)| {
                let width = (size.width as f32 * self.scale) as i32;
                let height = (size.height as f32 * self.scale) as i32;

                let page_top = doc_top_offset;
                let page_bottom = page_top + height;
                doc_top_offset = page_bottom;

                let page_rect = sk::IRect::new(0, page_top, width, page_bottom);

                let mut clip = sk::IRect::intersect(&page_rect, &doc_viewpoint)?;

                clip.offset((0, -page_top));

                Some((i, LogicalSize::new(width, height), clip))
            })
            .collect_vec();

        self.canvas().save();
        for (i, size, clip) in pages {
            let page = doc.get_page(i as u16)?;

            let page_width = size.width as i32;
            let page_height = size.height as i32;

            debug!("{i}, {page_width}, {page_height}, ({:?})", clip);

            let pdf_bitmap = page.render_with_config(
                &PdfRenderConfig::default()
                    .set_target_size(page_width, page_height)
                    .set_format(PdfBitmapFormat::BGRA),
            )?;

            let mut bitmap = sk::Bitmap::new();

            unsafe {
                bitmap.install_pixels(
                    &sk::ImageInfo::new(
                        (page_width, page_height),
                        sk::ColorType::BGRA8888,
                        sk::AlphaType::Opaque,
                        None,
                    ),
                    pdf_bitmap.as_bytes().as_ptr().cast_mut().cast::<c_void>(),
                    (page_width * 4) as usize,
                );
            }

            self.canvas().translate((0, -clip.top));

            self.canvas().save();

            self.canvas()
                .translate(((width as i32 - page_width) / 2 - 2, 0));

            self.canvas().draw_image(bitmap.as_image(), (0, 0), None);

            // self.draw_selection(&page, (page_width, page_height), sentence)?;

            self.canvas().restore();

            self.canvas().translate((0, page_height));
        }

        self.canvas().restore();

        Ok(())
    }

    fn redraw(&mut self) -> Result<(), anyhow::Error> {
        self.canvas().clear(sk::Color::GRAY);

        if let Some(doc) = self.doc.clone() {
            self.render_pdf(doc)?;
        }

        let _ = self.image_snapshot.send(self.surface.image_snapshot());
        Ok(())
    }

    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), anyhow::Error> {
        let PhysicalSize { width, height } = size.cast::<i32>();
        self.surface =
            sk::surfaces::raster_n32_premul((width, height)).context("create pdf surface")?;

        self.bitmap = PdfBitmap::empty(
            width,
            height,
            PdfBitmapFormat::BGRA,
            Pdf::get_unwrap().bindings(),
        )?;

        self.size = size;
        Ok(())
    }

    async fn handle_event(&mut self, e: PdfEvent) -> Result<bool, anyhow::Error> {
        Ok(match e {
            PdfEvent::DocChanged(doc) => {
                self.doc = Some(PdfDocument::new(Arc::try_unwrap(doc).unwrap()));
                true
            }
            PdfEvent::Scale(ScaleEvent {
                scale,
                mouse_point: _,
            }) => {
                self.scale = scale;
                true
            }
            PdfEvent::Scroll(ScrollEvent { left, top }) => {
                self.scroll_offset = PhysicalPosition::new(left, top);
                true
            }
            PdfEvent::Window(e) => match e {
                WindowEvent::Resized(size) => {
                    if size.width == 0 || size.height == 0 {
                        false
                    } else {
                        self.resize(size.into())?;
                        true
                    }
                }
                _ => false,
            },
        })
    }
    async fn handle_multi_event(&mut self, events: Vec<PdfEvent>) -> Result<(), anyhow::Error> {
        let mut need_redraw = false;
        for e in events {
            if self.handle_event(e).await? {
                need_redraw = true;
            }
        }

        if need_redraw {
            self.redraw()?;
        }
        Ok(())
    }

    async fn run_loop(
        mut self,
        mut event_receiver: broadcast::Receiver<PdfEvent>,
    ) -> Result<(), anyhow::Error> {
        while let Ok(e) = event_receiver.recv().await {
            let mut events = vec![e];
            while let Ok(e) = event_receiver.try_recv() {
                events.push(e);
            }
            self.handle_multi_event(events).await?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PdfViewer {
    image_snapshot: watch::Receiver<sk::Image>,
}

impl PdfViewer {
    pub async fn new(win: Arc<WGuiWindow>) -> Result<Self, anyhow::Error> {
        let loader = win.state::<PdfLoader>();

        let event_receiver = pdf_event_receiver(&loader, &win).await;

        let (renderer, image_snapshot) = PdfRenderer::new(win.inner_size()?)?;

        tokio::spawn(async move {
            if let Err(e) = renderer.run_loop(event_receiver).await {
                warn!("{:?}", e);
            }
        });

        Ok(Self { image_snapshot })
    }

    pub fn draw(&self, canvas: &mut sk::Canvas) -> Result<(), anyhow::Error> {
        let image = self.image_snapshot.borrow().clone();

        canvas.draw_image(image, (0, 0), None);
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum PdfEvent {
    DocChanged(Arc<Document>),
    Scale(ScaleEvent),
    Scroll(ScrollEvent),
    Window(WindowEvent),
}

async fn pdf_event_receiver(loader: &PdfLoader, win: &WGuiWindow) -> broadcast::Receiver<PdfEvent> {
    let (tx, rx) = broadcast::channel(64);
    {
        let tx = tx.clone();
        loader.set_doc_loaded_handler(move |doc| {
            let _ = tx.send(PdfEvent::DocChanged(Arc::new(doc)));
        });
    }

    {
        let tx = tx.clone();
        win.listen("pdf-event", move |e| {
            if let Some(payload) = e.payload() {
                match serde_json::from_str::<WPdfEvent>(payload) {
                    Ok(e) => match e {
                        WPdfEvent::Scale(e) => {
                            let _ = tx.send(PdfEvent::Scale(e));
                        }
                        WPdfEvent::Scroll(e) => {
                            let _ = tx.send(PdfEvent::Scroll(e));
                        }
                    },
                    Err(_) => {
                        warn!("{:?}", e);
                    }
                }
            }
        });
    }

    {
        let tx = tx.clone();
        win.on_window_event(move |e| {
            let _ = tx.send(PdfEvent::Window(e.clone()));
        });
    }

    rx
}
