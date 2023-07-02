use std::ops::Deref;

use futures::FutureExt;
use mapp::provider::Res;
use pdfium_render::prelude::*;
use send_wrapper::SendWrapper;

#[cfg(target_family = "wasm")]
use web_sys::Blob;

mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(catch)]
        pub async fn PDFiumModule() -> Result<JsValue, JsValue>;
    }
}

pub struct Pdf {
    inner: SendWrapper<Pdfium>,
}

impl Pdf {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
        ffi::PDFiumModule().await.unwrap();

        Ok(Res::new(Self::new()?))
    }

    fn new() -> Result<Self, anyhow::Error> {
        let bindings = Pdfium::bind_to_system_library()
            .map_err(|e| anyhow::anyhow!("Failed to load pdfium library: {}", e))?;

        Ok(Self {
            inner: SendWrapper::new(Pdfium::new(bindings)),
        })
    }

    #[cfg(target_family = "wasm")]
    pub async fn load_from_blob(
        self: &Res<Self>,
        blob: Blob,
        password: Option<&str>,
    ) -> Result<PdfDoc, anyhow::Error> {
        PdfDoc::load_from_blob(self.clone(), blob, password).await
    }
}

pub struct PdfDoc {
    doc: PdfDocument<'static>,
    _pdf: Res<Pdf>,
}

impl Deref for PdfDoc {
    type Target = PdfDocument<'static>;

    fn deref(&self) -> &Self::Target {
        &self.doc
    }
}

impl PdfDoc {
    #[cfg(target_family = "wasm")]
    async fn load_from_blob(
        pdf: Res<Pdf>,
        blob: Blob,
        password: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        let doc = pdf
            .inner
            .load_pdf_from_blob(blob, password)
            .await
            .map(|v| unsafe { std::mem::transmute(v) })
            .map_err(|e| anyhow::anyhow!("Failed to load pdf: {}", e))?;

        Ok(Self { doc, _pdf: pdf })
    }
}
