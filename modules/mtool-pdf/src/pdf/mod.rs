use mapp::provider::Res;
use ouroboros::self_referencing;
use pdfium_render::prelude::*;
use send_wrapper::SendWrapper;
use std::ops::Deref;

pub struct Pdf {
    inner: SendWrapper<Pdfium>,
}

impl Pdf {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
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
    pub async fn load(
        self: &Res<Self>,
        url: impl ToString,
        password: Option<&str>,
    ) -> Result<PdfDoc, anyhow::Error> {
        PdfDoc::load(self.clone(), url, password).await
    }

    // #[cfg(not(target_family = "wasm"))]
    // pub async fn load(
    //     self: &Res<Self>,
    //     path: &(impl AsRef<Path> + ?Sized),
    //     password: Option<&str>,
    // ) -> Result<PdfDoc, anyhow::Error> {
    //     PdfDoc::load(self.clone(), path, password).await
    // }
}

#[self_referencing]
pub struct PdfDoc {
    pdf: Res<Pdf>,
    password: Option<String>,
    #[borrows(pdf)]
    pdf_inner: &'this Pdfium,
    #[borrows(pdf_inner, password)]
    #[covariant]
    doc: PdfDocument<'this>,
}

impl PdfDoc {
    // #[cfg(target_family = "wasm")]
    async fn load(
        pdf: Res<Pdf>,
        url: impl ToString,
        password: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        use futures::FutureExt;
        use tokio::task::spawn_local;
        let url = url.to_string();

        Ok(spawn_local(
            PdfDocAsyncTryBuilder {
                pdf,
                password: password.map(|v| v.to_string()),
                pdf_inner_builder: |pdf| async move { Ok(pdf.inner.deref()) }.boxed_local(),
                doc_builder: |pdf, password| {
                    async move {
                        pdf.load_pdf_from_fetch(&url, password.as_deref())
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to load pdf {}: {}", url, e))
                    }
                    .boxed_local()
                },
            }
            .try_build(),
        )
        .await??)
    }

    // #[cfg(not(target_family = "wasm"))]
    // async fn load(
    //     pdf: Res<Pdf>,
    //     path: &str,
    //     password: Option<&str>,
    // ) -> Result<Self, anyhow::Error> {
    //     Ok(PdfDocAsyncSendTryBuilder {
    //         pdf,
    //         password: password.map(|v| v.to_string()),
    //         doc_builder: |pdf, password| {
    //             async move {
    //                 let pdf = pdf.inner.lock().await;
    //                 pdf.load_pdf_from_file(path, password.as_deref())
    //                     .context(format!("Loading pdf: {}", path))
    //             }
    //             .boxed()
    //         },
    //     }
    //     .try_build()
    //     .await?)
    // }
}
