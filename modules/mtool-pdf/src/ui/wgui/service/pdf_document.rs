use std::ops::Deref;

use itertools::Itertools;
use pdfium_render::prelude as pdfium;

use crate::ui::wgui::{PageInfo, PdfDocumentInfo};

pub struct PdfDocument {
    doc: pdfium::PdfDocument<'static>,
    info: PdfDocumentInfo,
}

unsafe impl Send for PdfDocument {}

impl Deref for PdfDocument {
    type Target = pdfium::PdfDocument<'static>;

    fn deref(&self) -> &Self::Target {
        &self.doc
    }
}

impl std::fmt::Debug for PdfDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document").finish()
    }
}

impl PdfDocument {
    pub(super) async fn new(doc: pdfium::PdfDocument<'static>) -> Result<Self, anyhow::Error> {
        let info = load_document_info(&doc).await?;

        Ok(Self { doc, info })
    }

    pub fn document_info(&self) -> &PdfDocumentInfo {
        &self.info
    }

    pub fn width(&self) -> usize {
        self.document_info().width()
    }

    pub fn height(&self) -> usize {
        self.document_info().height()
    }
}

async fn load_document_info(
    doc: &pdfium::PdfDocument<'static>,
) -> Result<PdfDocumentInfo, anyhow::Error> {
    let n_pages = doc.pages().len();
    let pages = (0..n_pages)
        .map(|i| {
            doc.page_size_by_index(i as usize).map(|size| PageInfo {
                width: (size.width().to_inches() * 96.) as usize,
                height: (size.height().to_inches() * 96.) as usize,
            })
        })
        .try_collect()?;
    Ok(PdfDocumentInfo { pages })
}
