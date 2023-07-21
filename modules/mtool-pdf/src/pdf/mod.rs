use anyhow::Context;
use async_trait::async_trait;
use mapp::prelude::*;
use mtool_core::ConfigStore;
use ouroboros::self_referencing;
use pdfium_render::prelude::*;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::Config;

pub struct Pdf {
    inner: Pdfium,
}

impl Deref for Pdf {
    type Target = Pdfium;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Pdf {
    async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self::new(&cs.get("pdf").await?)?))
    }

    fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let bindings = Pdfium::bind_to_library(&config.library)
            .map_err(|e| anyhow::anyhow!("Failed to load pdfium library: {}", e))?;

        Ok(Self {
            inner: Pdfium::new(bindings),
        })
    }

    pub fn load(
        self: &Res<Self>,
        path: &(impl AsRef<Path> + ?Sized),
        password: Option<&str>,
    ) -> Result<PdfDoc, anyhow::Error> {
        PdfDoc::load(self.clone(), path, password)
    }
}

#[self_referencing]
pub struct PdfDoc {
    pdf: Res<Pdf>,
    pub path: PathBuf,
    password: Option<String>,
    #[borrows(pdf)]
    pdf_inner: &'this Pdfium,
    #[borrows(pdf_inner, password)]
    #[not_covariant]
    pub doc: PdfDocument<'this>,
}

impl PdfDoc {
    fn load(
        pdf: Res<Pdf>,
        path: &(impl AsRef<Path> + ?Sized),
        password: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        PdfDocTryBuilder {
            pdf,
            path: path.as_ref().to_path_buf(),
            password: password.map(|v| v.to_string()),
            pdf_inner_builder: |pdf| Ok(pdf.deref()),
            doc_builder: |pdf, password| {
                pdf.load_pdf_from_file(path, password.as_deref())
                    .context(format!("Loading pdf: {}", path.as_ref().display()))
            },
        }
        .try_build()
    }

    pub fn page(&self, index: u16) -> Result<PdfPage, anyhow::Error> {
        self.with_doc(|doc| Ok(doc.pages().get(index)?))
    }
}

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(Pdf::construct);
        Ok(())
    }
}
