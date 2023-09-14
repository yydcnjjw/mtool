use std::path::PathBuf;

use anyhow::Context;
use mcloud_api::adobe;
use parking_lot::RwLock;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, Wry,
};
use tracing::{debug, warn};

use crate::{
    pdf::Pdf,
    ui::wgui::{PdfDocumentInfo, PdfFile},
    Config,
};

use super::PdfDocument;

type OnDocLoaded = Box<dyn Fn(PdfDocument) + Send + Sync>;

pub struct PdfLoader {
    cache_dir: PathBuf,
    on_doc_loaded: RwLock<Option<OnDocLoaded>>,
}

impl PdfLoader {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            on_doc_loaded: RwLock::default(),
        }
    }

    pub fn doc_loaded_handler<Handler>(&self, handler: Handler)
    where
        Handler: Fn(PdfDocument) + Send + Sync + 'static,
    {
        *self.on_doc_loaded.write() = Some(Box::new(handler));
    }

    async fn load(&self, file: PdfFile) -> Result<PdfDocumentInfo, anyhow::Error> {
        let doc = Pdf::get_unwrap()
            .load_pdf_from_file(&file.path, file.password.map(|p| p.leak() as &'static str))?;

        let doc = PdfDocument::new(
            doc,
            // Self::load_grobid_tei().await?,
            Self::load_adobe_structure(&file.path)
                .await
                .inspect_err(|e| {
                    warn!("{:?}", e);
                })
                .ok(),
        )
        .await?;
        let info = doc.document_info().clone();

        if let Some(on_doc_loaded) = self.on_doc_loaded.read().as_ref() {
            on_doc_loaded(doc);
        }

        Ok(info)
    }

    // async fn load_grobid_tei() -> Result<grobid::TEI, anyhow::Error> {
    //     let xml = tokio::fs::read_to_string("/home/yydcnjjw/.mtool/cache/nilsson1999.pdf.tei.xml")
    //         .await?;
    //     Ok(quick_xml::de::from_str(&xml)?)
    // }

    async fn load_adobe_structure(path: &str) -> Result<adobe::PdfStructure, anyhow::Error> {
        // "c:/Users/yydcnjjw/.mtool/cache/structuredData.json"
        let json = tokio::fs::read_to_string("c:/Users/yydcnjjw/.mtool/cache/test.json").await?;
        Ok(serde_json::from_str(&json)?)
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

pub fn init(config: &Config) -> TauriPlugin<Wry> {
    let cache_dir = config.cache_dir.clone();
    Builder::new("pdfloader")
        .setup(move |app, _| {
            app.manage(PdfLoader::new(cache_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_pdf])
        .build()
}
