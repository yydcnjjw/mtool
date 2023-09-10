use std::{ops::Deref, rc::Rc};

use pdfium_render::prelude as pdfium;
use skia_safe as sk;

pub struct PdfTextRange {
    pub page_index: u16,
    pub index: pdfium::PdfPageTextCharIndex,
    pub count: usize,
}

impl PdfTextRange {
    pub fn new(page_index: u16, index: pdfium::PdfPageTextCharIndex, count: usize) -> Self {
        Self {
            page_index,
            index,
            count,
        }
    }
}

pub struct PdfPageInner {
    inner: pdfium::PdfPage<'static>,
    index: u16,
    // sentences: Vec<TextRange>,
}

impl Deref for PdfPageInner {
    type Target = pdfium::PdfPage<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PdfPageInner {
    pub fn new(inner: pdfium::PdfPage<'static>, index: u16) -> Result<Self, anyhow::Error> {
        // let sentences = Self::parse_sentences(&inner)?;
        Ok(Self {
            inner, // sentences
            index,
        })
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn get_text_rects(
        &self,
        page_size: sk::ISize,
        text_range: &PdfTextRange,
    ) -> Result<Vec<sk::IRect>, anyhow::Error> {
        let bindings = self.bindings();
        let text_page = self.text()?;
        let text_page_handle = text_page.handle();

        let count = bindings.FPDFText_CountRects(
            *text_page_handle,
            text_range.index as i32,
            text_range.count as i32,
        );

        let page_region = sk::IRect::new(0, 0, page_size.width, page_size.height);

        let mut rects = Vec::with_capacity(count as usize);

        for i in 0..count {
            let (mut page_left, mut page_top, mut page_right, mut page_bottom) = (0., 0., 0., 0.);
            bindings.FPDFText_GetRect(
                *text_page_handle,
                i,
                &mut page_left,
                &mut page_top,
                &mut page_right,
                &mut page_bottom,
            );

            let (device_left, device_top) =
                self.page_to_device(&page_region, (page_left, page_top))?;
            let (device_right, device_bottom) =
                self.page_to_device(&page_region, (page_right, page_bottom))?;

            rects.push(sk::IRect::new(
                device_left,
                device_top,
                device_right,
                device_bottom,
            ));
        }

        Ok(rects)
    }

    pub fn page_to_device(
        &self,
        region: &sk::IRect,
        (page_x, page_y): (f64, f64),
    ) -> Result<(i32, i32), anyhow::Error> {
        let mut device_x = 0;
        let mut device_y = 0;
        if self.bindings().FPDF_PageToDevice(
            self.page_handle(),
            region.left,
            region.top,
            region.width(),
            region.height(),
            self.rotation()?.as_pdfium(),
            page_x,
            page_y,
            &mut device_x,
            &mut device_y,
        ) == 0
        {
            anyhow::bail!("({page_x},{page_y}) page to device failed at {region:?}");
        }

        Ok((device_x, device_y))
    }

    pub fn device_to_page(
        &self,
        region: &sk::IRect,
        (device_x, device_y): (i32, i32),
    ) -> Result<(f64, f64), anyhow::Error> {
        let mut page_x = 0.;
        let mut page_y = 0.;
        if self.bindings().FPDF_DeviceToPage(
            self.page_handle(),
            region.left,
            region.top,
            region.width(),
            region.height(),
            self.rotation()?.as_pdfium(),
            device_x,
            device_y,
            &mut page_x,
            &mut page_y,
        ) == 0
        {
            anyhow::bail!("({device_x},{device_y}) device to page failed at {region:?}");
        }

        Ok((page_x, page_y))
    }

    // pub fn sentences(&self) -> &Vec<TextRange> {
    //     &self.sentences
    // }

    // fn parse_sentences(inner: &pdfium::PdfPage) -> Result<Vec<TextRange>, anyhow::Error> {
    //     let mut sentences = Vec::new();
    //     let mut index = 0;
    //     for ch in inner.text()?.chars().iter() {
    //         if let Some(c) = ch.unicode_char() {
    //             if c == '.' {
    //                 sentences.push(TextRange::new(index, ch.index() - index));
    //                 index = ch.index();
    //             }
    //         }
    //     }
    //     Ok(sentences)
    // }
}

#[derive(Clone)]
pub struct PdfPage {
    inner: Rc<PdfPageInner>,
}

impl Deref for PdfPage {
    type Target = PdfPageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PdfPage {
    pub fn new(inner: pdfium::PdfPage<'static>, index: u16) -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: Rc::new(PdfPageInner::new(inner, index)?),
        })
    }
}
