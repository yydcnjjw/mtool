use std::{ops::Deref, rc::Rc};

use pdfium_render::prelude as pdfium;

pub struct TextRange {
    pub index: pdfium::PdfPageTextCharIndex,
    pub count: usize,
}

impl TextRange {
    fn new(index: pdfium::PdfPageTextCharIndex, count: usize) -> Self {
        Self { index, count }
    }
}

pub struct PdfPageInner {
    inner: pdfium::PdfPage<'static>,
    // sentences: Vec<TextRange>,
}

impl Deref for PdfPageInner {
    type Target = pdfium::PdfPage<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PdfPageInner {
    pub fn new(inner: pdfium::PdfPage<'static>) -> Result<Self, anyhow::Error> {
        // let sentences = Self::parse_sentences(&inner)?;
        Ok(Self {
            inner, // sentences
        })
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
    pub fn new(inner: pdfium::PdfPage<'static>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: Rc::new(PdfPageInner::new(inner)?),
        })
    }
}
