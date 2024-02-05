use std::any::type_name;

use anyhow::Context;
use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};
use mtool_wgui::{Builder, WGuiStage};
use tauri::{command, plugin::TauriPlugin, Manager, Runtime, State};
use tracing::warn;

use crate::dict::{ecdict, mdx, Backend};

use super::app::QueryResult;

#[command]
async fn dict_query(
    query: String,
    backend: Backend,
    injector: State<'_, Injector>,
) -> Result<QueryResult, serde_error::Error> {
    match dict_query_inner(query, backend, &injector)
        .await
        .context("dict query")
    {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!("{:?}", e);
            Err(serde_error::Error::new(&*e))
        }
    }
}

async fn dict_query_inner(
    query: String,
    backend: Backend,
    injector: &Injector,
) -> Result<QueryResult, anyhow::Error> {
    Ok(match backend {
        Backend::Mdx => QueryResult {
            template_id: type_name::<mdx::DictView>().to_string(),
            data: serde_json::to_value(
                injector.get::<Res<mdx::Dict>>().await?.query(&query).await,
            )?,
        },
        Backend::ECDict => QueryResult {
            template_id: type_name::<ecdict::DictView>().to_string(),
            data: serde_json::to_value(
                injector
                    .get::<Res<ecdict::Dict>>()
                    .await?
                    .query(&query)
                    .await?,
            )?,
        },
    })
}

fn init<R>(injector: Injector) -> TauriPlugin<R>
where
    R: Runtime,
{
    tauri::plugin::Builder::new("mtool-dict")
        .setup(|app, _| {
            app.manage(injector);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![dict_query])
        .build()
}

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule().add_once_task(WGuiStage::Setup, setup);
        Ok(())
    }
}

async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(init(injector))))?;
    Ok(())
}
