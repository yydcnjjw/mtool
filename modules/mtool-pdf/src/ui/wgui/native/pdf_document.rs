use std::{
    cell::{Ref, RefCell},
    collections::VecDeque,
    ops::Deref,
    rc::Rc,
};

use crate::ui::wgui::{service::PdfDocument as Document, PdfDocumentInfo};

use super::pdf_page::PdfPage;

pub struct DocumentInner {
    inner: Document,
    cached_pages: VecDeque<(u16, PdfPage)>,
}

impl Deref for DocumentInner {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DocumentInner {
    fn new(inner: Document) -> Self {
        Self {
            inner,
            cached_pages: VecDeque::new(),
        }
    }

    fn get_page(&mut self, index: u16) -> Result<PdfPage, anyhow::Error> {
        Ok(match self.cached_pages.iter().find(|(i, _)| *i == index) {
            Some((_, page)) => page.clone(),
            None => {
                static SIZE: usize = 10;

                if self.cached_pages.len() == SIZE {
                    self.cached_pages.pop_back();
                }

                self.cached_pages
                    .push_front((index, PdfPage::new(self.pages().get(index)?)?));

                self.cached_pages.front().unwrap().1.clone()
            }
        })
    }
}

#[derive(Clone)]
pub struct PdfDocument {
    inner: Rc<RefCell<DocumentInner>>,
}

impl PdfDocument {
    pub fn new(doc: Document) -> Self {
        Self {
            inner: Rc::new(RefCell::new(DocumentInner::new(doc))),
        }
    }

    pub fn width(&self) -> usize {
        self.inner.borrow().width()
    }

    pub fn height(&self) -> usize {
        self.inner.borrow().height()
    }

    pub fn get_page(&self, index: u16) -> Result<PdfPage, anyhow::Error> {
        self.inner.borrow_mut().get_page(index)
    }

    pub fn document_info(&self) -> Ref<'_, PdfDocumentInfo> {
        Ref::map(self.inner.borrow(), |doc| doc.document_info())
    }
}
