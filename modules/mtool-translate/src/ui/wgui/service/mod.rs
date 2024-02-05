use anyhow::Context;
use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};
use mtool_wgui::{Builder, WGuiStage};
use tauri::{command, plugin::TauriPlugin, Manager, Runtime, State};
use tracing::warn;

use crate::translator::{llama, openai, tencent, Backend, LanguageType, Translator};

#[derive(Default)]
pub struct Module {}

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

async fn text_translate_inner(
    input: String,
    source: LanguageType,
    target: LanguageType,
    backend: Backend,
    injector: &Injector,
) -> Result<String, anyhow::Error> {
    let translator: Res<dyn Translator + Send + Sync> = match backend {
        Backend::Tencent => injector.get::<Res<tencent::Translator>>().await?,
        Backend::Openai => injector.get::<Res<openai::Translator>>().await?,
        Backend::Llama => injector.get::<Res<llama::Translator>>().await?,
    };

    translator.text_translate(input, source, target).await
}

#[command]
async fn text_translate(
    input: String,
    source: LanguageType,
    target: LanguageType,
    backend: Backend,
    injector: State<'_, Injector>,
) -> Result<String, serde_error::Error> {
    match text_translate_inner(input, source, target, backend, &injector)
        .await
        .context("text translate")
    {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!("{:?}", e);
            Err(serde_error::Error::new(&*e))
        }
    }
}

fn init<R>(injector: Injector) -> TauriPlugin<R>
where
    R: Runtime,
{
    tauri::plugin::Builder::new("mtool-translate")
        .setup(|app, _| {
            app.manage(injector);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![text_translate])
        .build()
}
