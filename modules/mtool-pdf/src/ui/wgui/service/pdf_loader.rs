use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use futures::{future, Stream, TryStreamExt};
use mapp::prelude::*;
use mcloud_api::adobe;
use mtool_wgui::WindowDataBind;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, Wry,
};
use tokio::{
    fs,
    sync::{broadcast, oneshot, OnceCell},
};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tracing::debug;

use crate::{
    pdf::Pdf,
    storage::entity,
    ui::wgui::{native::PdfViewerWindow, PdfDocumentInfo, PdfFile},
    AdobeApiConfig, Config,
};

use super::PdfDocument;

#[derive(Debug, Clone)]
pub enum PdfLoadEvent {
    DocLoading,
    DocLoaded(Arc<PdfDocument>),
    DocStructureLoading,
    DocStructureLoaded,
}

struct PdfLoaderInner {
    tx: broadcast::Sender<(PdfFile, PdfLoadEvent)>,
    db: Res<DatabaseConnection>,

    adobe_api_cfg: AdobeApiConfig,
    adobe_api_cli: OnceCell<adobe::Client>,
}

impl PdfLoaderInner {
    fn new(adobe_api_cfg: AdobeApiConfig, db: Res<DatabaseConnection>) -> Arc<Self> {
        let (tx, _) = broadcast::channel(128);

        Arc::new(Self {
            tx,
            db,

            adobe_api_cfg,
            adobe_api_cli: OnceCell::new(),
        })
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

    fn send_event(&self, file: &PdfFile, e: PdfLoadEvent) {
        let _ = self.tx.send((file.clone(), e));
    }

    pub fn subscribe_event(&self) -> broadcast::Receiver<(PdfFile, PdfLoadEvent)> {
        self.tx.subscribe()
    }

    pub fn subscribe_event_with_file(
        &self,
        file: &PdfFile,
    ) -> impl Stream<Item = Result<(PdfFile, PdfLoadEvent), BroadcastStreamRecvError>> {
        let file = file.clone();
        BroadcastStream::new(self.subscribe_event())
            .try_filter(move |(f, _)| future::ready(f == &file))
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

    pub fn subscribe_event(&self) -> broadcast::Receiver<(PdfFile, PdfLoadEvent)> {
        self.inner.subscribe_event()
    }

    pub fn subscribe_event_with_file(
        &self,
        file: &PdfFile,
    ) -> impl Stream<Item = Result<(PdfFile, PdfLoadEvent), BroadcastStreamRecvError>> {
        self.inner.subscribe_event_with_file(file)
    }

    async fn load(&self, file: PdfFile) -> Result<Arc<PdfDocument>, anyhow::Error> {
        let (doc, doc_rx) = {
            let (tx, rx) = oneshot::channel();
            let this = self.inner.clone();
            let file = file.clone();
            (
                tokio::spawn(async move {
                    this.send_event(&file, PdfLoadEvent::DocLoading);
                    let doc = {
                        let file = file.clone();
                        tokio::task::spawn_blocking(move || {
                            Pdf::get_unwrap().load_pdf_from_file(
                                &file.path,
                                file.password.clone().map(|p| p.leak() as &'static str),
                            )
                        })
                        .await??
                    };

                    let doc = Arc::new(PdfDocument::new(doc).await?);
                    this.send_event(&file, PdfLoadEvent::DocLoaded(doc.clone()));
                    let _ = tx.send(doc.clone());
                    Ok::<_, anyhow::Error>(doc)
                }),
                rx,
            )
        };

        {
            let this = self.inner.clone();
            tokio::spawn(async move {
                this.send_event(&file, PdfLoadEvent::DocStructureLoading);
                let structure = this.load_adobe_structure(&file.path).await?;
                if let Ok(doc) = doc_rx.await {
                    doc.set_structure(structure);
                }
                Ok::<(), anyhow::Error>(())
            });
        }

        Ok(doc.await??)
    }
}

async fn load_pdf_inner(
    window: tauri::Window,
    loader: State<'_, PdfLoader>,
    file: PdfFile,
) -> Result<PdfDocumentInfo, anyhow::Error> {
    debug!("load file {:?}", file);
    let win = window
        .get_data::<PdfViewerWindow>()
        .context("PdfViewerWindow is not binded")?;
    let doc = loader.load(file).await?;
    win.render_document(doc.clone());
    Ok(doc.info().clone())
}

#[command]
async fn load_pdf(
    window: tauri::Window,
    loader: State<'_, PdfLoader>,
    file: PdfFile,
) -> Result<PdfDocumentInfo, serde_error::Error> {
    load_pdf_inner(window, loader, file)
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
