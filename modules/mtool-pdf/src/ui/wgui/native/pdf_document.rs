use std::{
    cell::{Ref, RefCell},
    collections::VecDeque,
    ops::Deref,
    rc::Rc, sync::Arc,
};

use mcloud_api::adobe;
use pdfium_render::prelude::{PdfPageIndex, PdfiumLibraryBindings};

use crate::ui::wgui::{
    service::PdfDocument as Document,
    PdfDocumentInfo,
};

use super::pdf_page::PdfPage;

pub struct DocumentInner {
    inner: Arc<Document>,
    cached_pages: VecDeque<(u16, PdfPage)>,
}

impl Deref for DocumentInner {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DocumentInner {
    fn new(inner: Arc<Document>) -> Self {
        Self {
            inner,
            cached_pages: VecDeque::new(),
        }
    }

    fn page_count(&self) -> u16 {
        self.pages().len()
    }

    fn get_page(&mut self, index: PdfPageIndex) -> Result<PdfPage, anyhow::Error> {
        Ok(match self.cached_pages.iter().find(|(i, _)| *i == index) {
            Some((_, page)) => page.clone(),
            None => {
                static SIZE: usize = 10;

                if self.cached_pages.len() == SIZE {
                    self.cached_pages.pop_back();
                }

                self.cached_pages
                    .push_front((index, PdfPage::new(self.pages().get(index)?, index)?));

                self.cached_pages.front().unwrap().1.clone()
            }
        })
    }

    fn get_page_paragraphs(&self, index: PdfPageIndex) -> Vec<adobe::Element> {
        self.inner
            .paragraphs()
            .filter(|p| p.page == Some(index) && p.bounds.is_some())
            .cloned()
            .collect()
    }
}

#[derive(Clone)]
pub struct PdfDocument {
    inner: Rc<RefCell<DocumentInner>>,
}

impl PdfDocument {
    pub fn new(doc: Arc<Document>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(DocumentInner::new(doc))),
        }
    }

    pub fn bindings(&self) -> &'static dyn PdfiumLibraryBindings {
        self.inner.borrow().bindings()
    }

    pub fn width(&self) -> usize {
        self.inner.borrow().width()
    }

    pub fn height(&self) -> usize {
        self.inner.borrow().height()
    }

    pub fn page_count(&self) -> u16 {
        self.inner.borrow().page_count()
    }

    pub fn get_page(&self, index: PdfPageIndex) -> Result<PdfPage, anyhow::Error> {
        self.inner.borrow_mut().get_page(index)
    }

    pub fn document_info(&self) -> Ref<'_, PdfDocumentInfo> {
        Ref::map(self.inner.borrow(), |doc| doc.info())
    }

    pub fn get_page_paragraphs(&self, index: PdfPageIndex) -> Vec<adobe::Element> {
        self.inner.borrow().get_page_paragraphs(index)
    }
}
