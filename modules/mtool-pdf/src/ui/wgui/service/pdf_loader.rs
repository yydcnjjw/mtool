use parking_lot::RwLock;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, Wry,
};

use crate::{
    pdf::Pdf,
    ui::wgui::{PdfDocumentInfo, PdfFile},
};

use super::PdfDocument;

type OnDocLoaded = Box<dyn Fn(PdfDocument) + Send + Sync>;

pub struct PdfLoader {
    on_doc_loaded: RwLock<Option<OnDocLoaded>>,
}

impl PdfLoader {
    pub fn new() -> Self {
        Self {
            on_doc_loaded: RwLock::default(),
        }
    }

    pub fn set_doc_loaded_handler<Handler>(&self, handler: Handler)
    where
        Handler: Fn(PdfDocument) + Send + Sync + 'static,
    {
        *self.on_doc_loaded.write() = Some(Box::new(handler));
    }

    async fn load(&self, file: PdfFile) -> Result<PdfDocumentInfo, anyhow::Error> {
        let doc = Pdf::get_unwrap()
            .load_pdf_from_file(&file.path, file.password.map(|p| p.leak() as &'static str))?;

        let doc = PdfDocument::new(doc).await?;
        let info = doc.document_info().clone();

        if let Some(on_doc_loaded) = self.on_doc_loaded.read().as_ref() {
            on_doc_loaded(doc);
        }

        Ok(info)
    }
}

#[command]
async fn load_pdf(
    loader: State<'_, PdfLoader>,
    file: PdfFile,
) -> Result<PdfDocumentInfo, serde_error::Error> {
    loader
        .load(file)
        .await
        .map_err(|e| serde_error::Error::new(&*e))
}

pub fn init() -> TauriPlugin<Wry> {
    Builder::new("pdfloader")
        .setup(move |app, _| {
            app.manage(PdfLoader::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_pdf])
        .build()
}
