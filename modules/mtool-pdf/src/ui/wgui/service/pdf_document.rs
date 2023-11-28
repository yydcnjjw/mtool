use std::ops::Deref;

use itertools::Itertools;
use mcloud_api::adobe;
use pdfium_render::prelude as pdfium;
use tokio::sync::OnceCell;

use crate::ui::wgui::{PageInfo, PdfDocumentInfo};

pub struct PdfDocument {
    doc: pdfium::PdfDocument<'static>,
    info: PdfDocumentInfo,
    structure: OnceCell<adobe::PdfStructure>,
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

        Ok(Self {
            doc,
            info,
            structure: OnceCell::new(),
        })
    }

    pub fn strucutre(&self) -> &OnceCell<adobe::PdfStructure> {
        &self.structure
    }

    pub fn info(&self) -> &PdfDocumentInfo {
        &self.info
    }

    pub fn width(&self) -> usize {
        self.info().width()
    }

    pub fn height(&self) -> usize {
        self.info().height()
    }

    pub fn paragraphs(&self) -> impl Iterator<Item = &adobe::Element> {
        self.structure.get().into_iter().flat_map(|adobe| {
            adobe
                .elements
                .iter()
                .filter(|elem| elem.path.starts_with("//Document/P"))
        })
    }

    // pub fn sentences(&self) -> impl Iterator<Item = &Sentence> {
    //     self.paragraphs().flat_map(|paragraph| paragraph.sentence())
    // }

    // pub fn paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
    //     self.tei
    //         .text
    //         .body
    //         .divs
    //         .iter()
    //         .flat_map(|div| div.paragraph())
    // }
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
