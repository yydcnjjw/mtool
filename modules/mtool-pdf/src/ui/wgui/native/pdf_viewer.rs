use std::{collections::HashMap, ffi::c_void, sync::Arc};

use anyhow::Context;
use itertools::Itertools;
use pdfium_render::prelude::*;
use skia_safe as sk;
use tauri::{PhysicalPosition, PhysicalSize, WindowEvent};
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{debug, trace, warn};

use super::{
    pdf_document::PdfDocument,
    pdf_page::{PdfPage, PdfTextRange},
};
use crate::{
    pdf::Pdf,
    ui::wgui::{
        event::{MouseEvent, PageInfo, Position, ScaleEvent, ScrollEvent, WMouseEvent, WPdfEvent},
        service::PdfLoadEvent,
    },
};

struct PdfViewerInner {
    doc: Option<PdfDocument>,

    surface: sk::Surface,

    pdf_bitmap: PdfBitmap<'static>,

    image_snapshot: watch::Sender<sk::Image>,

    selections: HashMap<PdfPageIndex, Vec<PdfTextRange>>,

    viewpoint: PhysicalSize<u32>,
    scale: f32,
    scroll_offset: PhysicalPosition<i32>,

    mouse_state: MouseState,
}

unsafe impl Send for PdfViewerInner {}

impl PdfViewerInner {
    fn new(
        viewpoint: PhysicalSize<u32>,
    ) -> Result<(Self, watch::Receiver<sk::Image>), anyhow::Error> {
        let PhysicalSize { width, height } = viewpoint.cast::<i32>();
        let mut surface =
            sk::surfaces::raster_n32_premul((width, height)).context("create pdf surface")?;

        let (image_snapshot, rx) = watch::channel(surface.image_snapshot());

        Ok((
            Self {
                doc: None,
                surface,
                pdf_bitmap: PdfBitmap::empty(
                    width,
                    height,
                    PdfBitmapFormat::BGRA,
                    Pdf::get_unwrap().bindings(),
                )?,
                image_snapshot,

                selections: HashMap::new(),

                viewpoint,
                scale: 1.,
                scroll_offset: PhysicalPosition::new(0, 0),
                mouse_state: MouseState::default(),
            },
            rx,
        ))
    }

    #[inline]
    fn canvas(&mut self) -> &sk::Canvas {
        self.surface.canvas()
    }

    fn draw_selection(
        &mut self,
        page: &PdfPage,
        page_size: sk::ISize,
    ) -> Result<(), anyhow::Error> {
        let text_ranges = match self.selections.get(&page.index()) {
            Some(v) => v,
            None => return Ok(()),
        };
        
        let mut highlight_rects = Vec::new();

        for text_range in text_ranges {
            let mut line_rect = sk::IRect::new_empty();

            for rect in page.get_text_rects(page_size, text_range)? {
                if line_rect.is_empty() {
                    line_rect = rect;
                } else {
                    // NOTE: if IRect is same, return false
                    // therefore we need to make the horizontal size different
                    if sk::IRect::intersects(
                        &sk::IRect::new(0, line_rect.top, i32::MAX, line_rect.bottom),
                        &sk::IRect::new(i32::MIN, rect.top, i32::MAX, rect.bottom),
                    ) {
                        line_rect = sk::IRect::join(&line_rect, &rect);
                    } else {
                        highlight_rects.push(line_rect);
                        line_rect = rect;
                    }
                }
            }
            highlight_rects.push(line_rect);
        }

        let mut paint = sk::Paint::new(sk::Color4f::from(sk::Color::from_rgb(153, 193, 218)), None);
        paint.set_blend_mode(sk::BlendMode::Multiply);
        for rect in highlight_rects {
            self.canvas().draw_irect(rect, &paint);
        }

        Ok(())
    }

    fn draw_pargraphs(
        &mut self,
        doc: &PdfDocument,
        page: &PdfPage,
        page_size: sk::ISize,
    ) -> Result<(), anyhow::Error> {
        let page_region = sk::IRect::new(0, 0, page_size.width, page_size.height);

        let mut paint = sk::Paint::new(sk::Color4f::from(sk::Color::BLUE), None);
        paint.set_stroke(true);

        for paragraph in doc.get_page_paragraphs(page.index()) {
            let [left, top, right, bottom] = paragraph.bounds.unwrap();

            let (left, top) = page.page_to_device(&page_region, (left, top))?;
            let (right, bottom) = page.page_to_device(&page_region, (right, bottom))?;

            self.canvas()
                .draw_irect(sk::IRect::new(left, top, right, bottom), &paint);
        }

        Ok(())
    }

    fn render_pdf(&mut self, doc: PdfDocument) -> Result<(), anyhow::Error> {
        let PhysicalSize {
            width: viewpoint_width,
            height: viewpoint_height,
        } = self.viewpoint;
        let (doc_width, _doc_height) = self.size_with_scale(doc.width(), doc.height());

        let scroll_offset = self.scroll_offset;

        let doc_viewpoint = sk::IRect::new(
            scroll_offset.x,
            scroll_offset.y,
            scroll_offset.x + (viewpoint_width as i32).min(doc_width),
            scroll_offset.y + viewpoint_height as i32,
        );

        trace!(
            "viewpoint={:?} doc_viewpoint={:?}",
            self.viewpoint,
            doc_viewpoint
        );

        let mut doc_top_offset = 0;

        let pages = doc
            .document_info()
            .pages
            .iter()
            .enumerate()
            .filter_map(|(i, size)| {
                let (page_width, page_height) = self.size_with_scale(size.width, size.height);

                let page_top = doc_top_offset;
                let page_bottom = page_top + page_height;
                doc_top_offset = page_bottom;

                let page_rect = sk::IRect::new(0, page_top, page_width, page_bottom);

                let mut clip = sk::IRect::intersect(&page_rect, &doc_viewpoint)?;

                clip.offset((0, -page_top));

                Some((i as u16, sk::ISize::new(page_width, page_height), clip))
            })
            .collect_vec();

        self.canvas().save();

        // TODO why -8?
        self.canvas().translate((
            ((viewpoint_width as i32 - doc_viewpoint.width() - 8) / 2),
            0,
        ));

        for (i, page_size, clip) in pages {
            let page = doc.get_page(i)?;

            let page_width = page_size.width;
            let page_height = page_size.height;

            trace!("{i}, {page_width}, {page_height}, ({:?})", clip);

            let mut pdf_bitmap = PdfBitmap::empty(
                page_width,
                page_height,
                PdfBitmapFormat::BGRA,
                doc.bindings(),
            )?;

            page.render_into_bitmap_with_config(
                &mut pdf_bitmap,
                &PdfRenderConfig::default()
                    .set_target_size(page_width, page_height)
                    .use_lcd_text_rendering(true)
                    .clear_before_rendering(true)
                    .render_form_data(false)
                    .render_annotations(true)
                    .set_format(PdfBitmapFormat::BGRA),
            )?;

            let mut bitmap = sk::Bitmap::new();

            unsafe {
                bitmap.install_pixels(
                    &sk::ImageInfo::new(
                        (clip.width(), clip.height()),
                        sk::ColorType::BGRA8888,
                        sk::AlphaType::Opaque,
                        None,
                    ),
                    {
                        let p = pdf_bitmap.as_bytes().as_ptr();
                        p.wrapping_add(
                            (page_width as i32 * 4 * clip.top() + scroll_offset.x * 4) as usize,
                        )
                        .cast_mut()
                        .cast::<c_void>()
                    },
                    (page_width * 4) as usize,
                );
            }

            {
                self.canvas().save();

                self.canvas().draw_image(bitmap.as_image(), (0, 0), None);

                {
                    self.canvas().save();
                    self.canvas().translate((-clip.left, -clip.top));
                    self.draw_selection(&page, page_size)?;
                    self.draw_pargraphs(&doc, &page, page_size)?;
                    self.canvas().restore();
                }

                self.canvas().restore();

                self.canvas().translate((0, clip.height()));
            }
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

    fn size_with_scale(&self, width: usize, height: usize) -> (i32, i32) {
        (
            (width as f32 * self.scale).round() as i32,
            (height as f32 * self.scale).round() as i32,
        )
    }

    fn extract_text(&self) -> () {}

    #[allow(unused)]
    fn highlight_sentence(
        &mut self,
        page_index: PdfPageIndex,
        position: sk::IPoint,
    ) -> Result<bool, anyhow::Error> {
        self.selections.clear();

        Ok(match self.doc.clone() {
            Some(doc) => {
                let page = doc.get_page(page_index)?;
                let PageInfo { width, height } = doc
                    .document_info()
                    .pages
                    .get(page_index as usize)
                    .context(format!("page is not exist: {page_index}"))?
                    .clone();

                let (page_width, page_height) = self.size_with_scale(width, height);

                let pos = page.device_to_page(
                    &sk::IRect::new(0, 0, page_width, page_height),
                    (position.x, position.y),
                )?;

                let (tolerance_width, tolerance_height) = (5, 5);

                let text_page = page.text()?;
                let chars = text_page.chars();
                if let Some(ch) = chars.get_char_near_point(
                    PdfPoints::new(pos.0 as f32),
                    PdfPoints::new(tolerance_width as f32),
                    PdfPoints::new(pos.1 as f32),
                    PdfPoints::new(tolerance_height as f32),
                ) {
                    let begin = match (0..ch.index()).rev().find(|ch| match chars.get(*ch) {
                        Ok(ch) => ch.unicode_char() == Some('.'),
                        Err(_) => false,
                    }) {
                        Some(ch) => ch + 1,
                        None => 0,
                    };

                    let end = match (ch.index()..chars.len()).find(|ch| match chars.get(*ch) {
                        Ok(ch) => ch.unicode_char() == Some('.'),
                        Err(_) => false,
                    }) {
                        Some(ch) => ch + 1,
                        None => {
                            if page_index + 1 != doc.page_count() {
                                let page = doc.get_page(page_index + 1)?;
                                let text_page = page.text()?;
                                let end = match text_page
                                    .chars()
                                    .iter()
                                    .find_or_last(|ch| ch.unicode_char() == Some('.'))
                                {
                                    Some(ch) => ch.index() + 1,
                                    None => text_page.chars().len(),
                                };
                                self.selections
                                    .entry(page_index + 1)
                                    .or_default()
                                    .push(PdfTextRange::new(page_index, 0, end));
                            }
                            chars.len()
                        }
                    };

                    self.selections
                        .entry(page_index)
                        .or_default()
                        .push(PdfTextRange::new(page_index, begin, end - begin));
                }
                true
            }
            None => false,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), anyhow::Error> {
        let PhysicalSize { width, height } = size.cast::<i32>();
        self.surface =
            sk::surfaces::raster_n32_premul((width, height)).context("create pdf surface")?;

        self.pdf_bitmap = PdfBitmap::empty(
            width,
            height,
            PdfBitmapFormat::BGRA,
            Pdf::get_unwrap().bindings(),
        )?;

        self.viewpoint = size;
        Ok(())
    }

    fn get_doc(&self) -> Result<PdfDocument, anyhow::Error> {
        self.doc.clone().context("Document is not loaded")
    }

    fn get_page_char_index(
        &self,
        page: &PdfPage,
        offset: PhysicalPosition<i32>,
    ) -> Result<Option<PdfPageTextCharIndex>, anyhow::Error> {
        let page_index = page.index() as usize;
        let PageInfo { width, height } = self
            .get_doc()?
            .document_info()
            .pages
            .get(page_index)
            .context(format!("page is not exist: {page_index}"))?
            .clone();

        let (page_width, page_height) = self.size_with_scale(width, height);

        let pos = page.device_to_page(
            &sk::IRect::new(0, 0, page_width, page_height),
            (offset.x, offset.y),
        )?;

        let (tolerance_width, tolerance_height) = (5, 5);

        let text_page = page.text()?;
        let chars = text_page.chars();
        Ok(chars
            .get_char_near_point(
                PdfPoints::new(pos.0 as f32),
                PdfPoints::new(tolerance_width as f32),
                PdfPoints::new(pos.1 as f32),
                PdfPoints::new(tolerance_height as f32),
            )
            .map(|char| char.index()))
    }

    fn get_page_text_range(
        &self,
        page_index1: u16,
        offset1: PhysicalPosition<i32>,
        page_index2: u16,
        offset2: PhysicalPosition<i32>,
    ) -> Result<Option<Vec<PdfTextRange>>, anyhow::Error> {
        let doc = self.get_doc()?;

        let is_same_page = page_index1 == page_index2;

        let page1 = doc.get_page(page_index1)?;
        let Some(char1) = self.get_page_char_index(&page1, offset1)? else {
            return Ok(None);
        };

        let page2 = if is_same_page {
            page1.clone()
        } else {
            doc.get_page(page_index2)?
        };
        let Some(char2) = self.get_page_char_index(&page2, offset2)? else {
            return Ok(None);
        };

        let mut ranges = Vec::new();
        if is_same_page {
            let count = char1.abs_diff(char2);
            let index = char1.min(char2);
            ranges.push(PdfTextRange::new(page_index1, index, count));
        } else {
            let ((first_page, first_page_char), (second_page, second_page_char)) =
                if page_index1 < page_index2 {
                    ((page1, char1), (page2, char2))
                } else {
                    ((page2, char2), (page1, char1))
                };

            {
                let count = first_page.text()?.chars().len() - first_page_char;
                let index = first_page_char;
                ranges.push(PdfTextRange::new(first_page.index(), index, count));
            }

            {
                let count = second_page_char;
                let index = 0;
                ranges.push(PdfTextRange::new(second_page.index(), index, count));
            }
        }
        Ok(Some(ranges))
    }

    fn get_selections(&mut self, page_index: u16) -> &mut Vec<PdfTextRange> {
        self.selections.entry(page_index).or_default()
    }

    fn handle_mouse_event(&mut self, page_index: u16, e: MouseEvent) -> bool {
        debug!("{}: {:?}", page_index, e);
        match e {
            MouseEvent::Up(_) => {
                self.mouse_state.pressed = None;
            }
            MouseEvent::Down(e) => {
                self.selections.clear();
                self.mouse_state.pressed = Some(PressState {
                    page: page_index,
                    pos: PhysicalPosition::new(e.offset.x, e.offset.y),
                });
            }
            MouseEvent::Move(e) => {
                if let Some(pressed) = self.mouse_state.pressed.clone() {
                    match self.get_page_text_range(
                        pressed.page,
                        pressed.pos,
                        page_index,
                        PhysicalPosition::new(e.offset.x, e.offset.y),
                    ) {
                        Ok(Some(ranges)) => {
                            self.selections.clear();
                            for range in ranges {
                                self.get_selections(range.page_index).push(range);
                            }
                        }
                        Err(e) => warn!("{:?}", e),
                        _ => {}
                    }
                } else {
                    return false
                }
            }
        }

        true
    }

    async fn handle_event(&mut self, e: PdfEvent) -> Result<bool, anyhow::Error> {
        Ok(match e {
            PdfEvent::PdfLoad(e) => match e {
                PdfLoadEvent::DocLoaded(doc) => {
                    self.doc = Some(PdfDocument::new(Arc::new(doc)));
                    true
                }
                PdfLoadEvent::DocStructureLoaded(structure) => {
                    if let Some(doc) = &self.doc {
                        doc.set_structure(structure);
                    }
                    true
                }
                _ => false,
            },
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
            PdfEvent::WGui(e) => match e {
                WPdfEvent::Scale(ScaleEvent {
                    scale,
                    mouse_point: _,
                }) => {
                    self.scale = scale;
                    true
                }
                WPdfEvent::Scroll(ScrollEvent { left, top }) => {
                    self.scroll_offset = PhysicalPosition::new(left, top);
                    true
                }
                WPdfEvent::Mouse { page_index, e } => self.handle_mouse_event(page_index, e),
            },
            PdfEvent::ExtractText { tx: _ } => true,
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
        mut event_receiver: mpsc::UnboundedReceiver<PdfEvent>,
    ) -> Result<(), anyhow::Error> {
        while let Some(e) = event_receiver.recv().await {
            let mut events = vec![e];
            while let Ok(e) = event_receiver.try_recv() {
                events.push(e);
            }
            self.handle_multi_event(events).await?;
        }
        Ok(())
    }
}

pub struct PdfViewer {
    image_snapshot: watch::Receiver<sk::Image>,
    event_sender: mpsc::UnboundedSender<PdfEvent>,
}

impl PdfViewer {
    pub async fn new(viewpoint: PhysicalSize<u32>) -> Result<Self, anyhow::Error> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let (renderer, image_snapshot) = PdfViewerInner::new(viewpoint)?;

        tokio::spawn(async move {
            if let Err(e) = renderer.run_loop(event_receiver).await {
                warn!("{:?}", e);
            }
        });

        Ok(Self {
            image_snapshot,
            event_sender,
        })
    }

    pub fn draw(&self, canvas: &sk::Canvas) -> Result<(), anyhow::Error> {
        let image = self.image_snapshot.borrow().clone();

        canvas.draw_image(image, (0, 0), None);
        Ok(())
    }

    pub fn notify_event(&self, e: PdfEvent) {
        let _ = self.event_sender.send(e);
    }

    pub async fn extract_text() {
        
    }
}

#[derive(Debug)]
pub enum PdfEvent {
    PdfLoad(PdfLoadEvent),

    Window(WindowEvent),

    WGui(WPdfEvent),

    ExtractText { tx: oneshot::Sender<String> },
}

#[derive(Debug, Clone)]
struct PressState {
    page: PdfPageIndex,
    pos: PhysicalPosition<i32>,
}

#[derive(Debug, Default)]
struct MouseState {
    pressed: Option<PressState>,
}
