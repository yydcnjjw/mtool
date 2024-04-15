use mapp::prelude::*;
use mtool_wgui::Builder;
use tauri::{command, plugin::TauriPlugin, Manager, Runtime, State};
use tracing::warn;

use crate::translator::{llama, openai, tencent, Backend, LanguageType, Translator};

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
    text_translate_inner(input, source, target, backend, &injector)
        .await
        .inspect_err(|e| warn!("{:?}", e))
        .map_err(|e| serde_error::Error::new(&*e))
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

pub async fn setup(builder: Res<Builder>, injector: Injector) -> Result<(), anyhow::Error> {
    builder.setup(|builder| Ok(builder.plugin(init(injector))))?;
    Ok(())
}
