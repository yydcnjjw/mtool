use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use mapp::prelude::*;
use mcloud_api::adobe::{self, PdfStructure};
use mtool_wgui::WindowDataBind;
use sea_orm::*;
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

        let adobe = match entity::adobe::Entity::find_by_id(md5sum.clone())
            .one(self.db.deref())
            .await?
        {
            Some(v) => v,
            None => {
                entity::adobe::Entity::insert(entity::adobe::ActiveModel {
                    id: Set(md5sum),
                    media_type: Set("application/pdf".into()),
                    ..Default::default()
                })
                .exec_with_returning(self.db.deref())
                .await?
            }
        };

        let AdobeApiConfig {
            url,
            client_id,
            key,
        } = &self.adobe_api_cfg;

        let cli = self
            .adobe_api_cli
            .get_or_try_init(|| async move { adobe::Client::new(url, client_id, key).await })
            .await?;

        let (asset_id, upload_uri) = if adobe.state == entity::adobe::State::GetAssetId {
            let (asset_id, upload_uri) = cli.get_asset_id(&adobe.media_type).await?;

            entity::adobe::Entity::update(entity::adobe::ActiveModel {
                id: Set(adobe.id.clone()),
                asset_id: Set(Some(asset_id.clone())),
                upload_uri: Set(Some(upload_uri.clone())),
                state: Set(entity::adobe::State::Upload),
                ..Default::default()
            })
            .exec(self.db.deref())
            .await?;

            (asset_id, upload_uri)
        } else {
            (adobe.asset_id.unwrap(), adobe.upload_uri.unwrap())
        };

        if adobe.state == entity::adobe::State::Upload {
            cli.upload_asset(&adobe.media_type, &upload_uri, file)
                .await?;

            entity::adobe::Entity::update(entity::adobe::ActiveModel {
                id: Set(adobe.id.clone()),
                state: Set(entity::adobe::State::ExtractPdf),
                ..Default::default()
            })
            .exec(self.db.deref())
            .await?;
        }

        Ok(if adobe.state == entity::adobe::State::ExtractPdf {
            let structure = cli.extract_pdf(asset_id).await?;

            entity::adobe::Entity::update(entity::adobe::ActiveModel {
                id: Set(adobe.id.clone()),
                state: Set(entity::adobe::State::End),
                structure: Set(Some(entity::PdfStructure::new(structure.clone()))),
                ..Default::default()
            })
            .exec(self.db.deref())
            .await?;

            structure
        } else {
            adobe.structure.unwrap().into()
        })
    }
}

pub struct PdfLoader {
    inner: Arc<PdfLoaderInner>,
}

pub struct PdfLoadWorker {
    _file: PdfFile,
    _set: JoinSet<Result<(), anyhow::Error>>,
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
            _file: file,
            _set: set,
            rx: Some(rx),
        })
    }
}

async fn load_pdf_inner(
    window: tauri::WebviewWindow,
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
    window: tauri::WebviewWindow,
    loader: State<'_, PdfLoader>,
    file: PdfFile,
) -> Result<(), serde_error::Error> {
    load_pdf_inner(window, loader, file)
        .await
        .map_err(|e| serde_error::Error::new(&*e))
}

pub fn init(config: Config, db: Res<DatabaseConnection>) -> TauriPlugin<Wry> {
    Builder::new("mtool-pdf")
        .setup(move |app, _| {
            app.manage(PdfLoader::new(config.adobe_api, db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_pdf])
        .build()
}
