use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use mapp::prelude::*;
use mcloud_api::adobe::{self, PdfStructure};
use mtool_wgui::WindowDataBind;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, State, Wry,
};
use tokio::{
    fs,
    sync::{mpsc, OnceCell},
    task::JoinSet,
};
use tracing::debug;

use crate::{
    pdf::Pdf,
    storage::entity,
    ui::wgui::{event::PdfFile, native::PdfViewerWindow},
    AdobeApiConfig, Config,
};

use super::PdfDocument;

#[derive(Debug)]
pub enum PdfLoadEvent {
    DocLoading,
    DocLoaded(PdfDocument),
    DocStructureLoading,
    DocStructureLoaded(PdfStructure),
}

struct PdfLoaderInner {
    db: Res<DatabaseConnection>,

    adobe_api_cfg: AdobeApiConfig,
    adobe_api_cli: OnceCell<adobe::Client>,
}

impl PdfLoaderInner {
    fn new(adobe_api_cfg: AdobeApiConfig, db: Res<DatabaseConnection>) -> Arc<Self> {
        Arc::new(Self {
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
}

pub struct PdfLoader {
    inner: Arc<PdfLoaderInner>,
}

pub struct PdfLoadWorker {
    file: PdfFile,
    set: JoinSet<Result<(), anyhow::Error>>,
    rx: Option<mpsc::Receiver<PdfLoadEvent>>,
}

impl PdfLoadWorker {
    pub fn subscribe(&mut self) -> Option<mpsc::Receiver<PdfLoadEvent>> {
        self.rx.take()
    }
}

impl PdfLoader {
    pub fn new(adobe_api_cfg: AdobeApiConfig, db: Res<DatabaseConnection>) -> Self {
        Self {
            inner: PdfLoaderInner::new(adobe_api_cfg, db),
        }
    }

    async fn load(&self, file: PdfFile) -> Result<PdfLoadWorker, anyhow::Error> {
        let mut set = JoinSet::new();
        let (tx, rx) = mpsc::channel(4);
        {
            let file = file.clone();
            let tx = tx.clone();
            set.spawn(async move {
                let _ = tx.send(PdfLoadEvent::DocLoading).await;
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

                let doc = PdfDocument::new(doc).await?;
                let _ = tx.send(PdfLoadEvent::DocLoaded(doc)).await;
                Ok::<(), anyhow::Error>(())
            });
        };

        {
            let this = self.inner.clone();
            let file = file.clone();
            let tx = tx.clone();
            set.spawn(async move {
                let _ = tx.send(PdfLoadEvent::DocStructureLoading).await;
                let structure = this.load_adobe_structure(&file.path).await?;
                let _ = tx.send(PdfLoadEvent::DocStructureLoaded(structure)).await;
                Ok::<(), anyhow::Error>(())
            });
        }

        Ok(PdfLoadWorker {
            file,
            set,
            rx: Some(rx),
        })
    }
}

async fn load_pdf_inner(
    window: tauri::Window,
    loader: State<'_, PdfLoader>,
    file: PdfFile,
) -> Result<(), anyhow::Error> {
    debug!("load file {:?}", file);
    let win = window
        .get_data::<PdfViewerWindow>()
        .context("PdfViewerWindow is not binded")?;
    win.set_pdf_loader(loader.load(file).await?);
    Ok(())
}

#[command]
async fn load_pdf(
    window: tauri::Window,
    loader: State<'_, PdfLoader>,
    file: PdfFile,
) -> Result<(), serde_error::Error> {
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
