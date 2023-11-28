use std::{ops::Deref, sync::Arc};

use mapp::prelude::*;
use mcloud_api::adobe;
use parking_lot::RwLock;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, Wry,
};
use tokio::{
    fs,
    sync::{mpsc, OnceCell},
};

use crate::{
    pdf::Pdf,
    storage::entity,
    ui::wgui::{PdfDocumentInfo, PdfFile},
    AdobeApiConfig, Config,
};

use super::PdfDocument;

pub enum PdfLoadEvent {
    DocLoaded(PdfFile, Arc<PdfDocument>),
    DocStructureLoaded(PdfFile),
    Err(anyhow::Error),
}

type LoadEventHandler = Option<Box<dyn Fn(PdfLoadEvent) + Send + Sync>>;

struct PdfLoaderInner {
    tx: mpsc::UnboundedSender<PdfLoadEvent>,
    db: Res<DatabaseConnection>,
    load_event_handler: RwLock<LoadEventHandler>,

    adobe_api_cfg: AdobeApiConfig,
    adobe_api_cli: OnceCell<adobe::Client>,
}

impl PdfLoaderInner {
    fn new(adobe_api_cfg: AdobeApiConfig, db: Res<DatabaseConnection>) -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        let this = Arc::new(Self {
            tx,
            db,
            load_event_handler: RwLock::new(None),

            adobe_api_cfg,
            adobe_api_cli: OnceCell::new(),
        });

        Self::handle_event(this.clone(), rx);
        this
    }

    fn handle_event(this: Arc<Self>, mut rx: mpsc::UnboundedReceiver<PdfLoadEvent>) {
        tokio::spawn(async move {
            while let Some(e) = rx.recv().await {
                let handler = this.load_event_handler.read();
                if let Some(handler) = handler.as_ref() {
                    handler(e);
                }
            }
        });
    }

    async fn load_adobe_structure(&self, path: &str) -> Result<adobe::PdfStructure, anyhow::Error> {
        let file = fs::read(path).await?;
        let md5sum = md5::compute(&file).to_vec();
        match entity::adobe::Entity::find_by_id(md5sum.clone())
            .one(self.db.deref())
            .await?
        {
            Some(v) => Ok(v.structure.into()),
            None => {
                let AdobeApiConfig {
                    url,
                    client_id,
                    key,
                } = &self.adobe_api_cfg;
                match self
                    .adobe_api_cli
                    .get_or_try_init(
                        || async move { adobe::Client::new(url, client_id, key).await },
                    )
                    .await
                {
                    Ok(cli) => {
                        let asset_id = cli.upload_asset("application/pdf", file).await?;
                        let structure = cli.extract_pdf(asset_id).await?;
                        entity::adobe::Entity::insert(entity::adobe::ActiveModel {
                            id: ActiveValue::Set(md5sum),
                            structure: ActiveValue::Set(entity::adobe::PdfStructure::new(
                                structure.clone(),
                            )),
                        })
                        .exec(self.db.deref())
                        .await?;
                        Ok(structure)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn send_event(&self, e: PdfLoadEvent) {
        let _ = self.tx.send(e);
    }
}

pub struct PdfLoader {
    inner: Arc<PdfLoaderInner>,
}

impl PdfLoader {
    pub fn new(adobe_api_cfg: AdobeApiConfig, db: Res<DatabaseConnection>) -> Self {
        Self {
            inner: PdfLoaderInner::new(adobe_api_cfg, db),
        }
    }

    pub fn on_load_event<Handler>(&self, handler: Handler)
    where
        Handler: Fn(PdfLoadEvent) + Send + Sync + 'static,
    {
        *self.inner.load_event_handler.write() = Some(Box::new(handler))
    }

    async fn load(&self, file: PdfFile) -> Result<PdfDocumentInfo, anyhow::Error> {
        let doc = Pdf::get_unwrap().load_pdf_from_file(
            &file.path,
            file.password.clone().map(|p| p.leak() as &'static str),
        )?;

        let doc = Arc::new(PdfDocument::new(doc).await?);

        let info = doc.info().clone();

        self.inner
            .send_event(PdfLoadEvent::DocLoaded(file.clone(), doc.clone()));

        {
            let this = self.inner.clone();
            tokio::spawn(async move {
                let path = file.path.clone();
                let e = {
                    let this = this.clone();
                    match doc
                        .strucutre()
                        .get_or_try_init(
                            move || async move { this.load_adobe_structure(&path).await },
                        )
                        .await
                    {
                        Ok(_) => PdfLoadEvent::DocStructureLoaded(file),
                        Err(e) => PdfLoadEvent::Err(e),
                    }
                };
                this.send_event(e);
            });
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

pub fn init(config: Config, db: Res<DatabaseConnection>) -> TauriPlugin<Wry> {
    Builder::new("pdfloader")
        .setup(move |app, _| {
            app.manage(PdfLoader::new(config.adobe_api, db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_pdf])
        .build()
}
