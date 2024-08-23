use std::any::type_name;

use anyhow::Context;
use mapp::prelude::*;
use mtool_wgui::Builder;
use tauri::{command, plugin::TauriPlugin, Runtime, State};
use tracing::warn;

use crate::{
    dict::{ecdict, mdx, Backend},
    ui::wgui::generic::QueryResult,
};

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

fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    tauri::plugin::Builder::new("mtool-dict")
        .setup(|_, _| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![dict_query])
        .build()
}

pub async fn setup(builder: Res<Builder>) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(init())))?;
    Ok(())
}
