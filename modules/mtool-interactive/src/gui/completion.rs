use std::sync::Arc;

use anyhow::Context as _;
use async_trait::async_trait;
use futures::lock::Mutex;
use mapp::provider::Res;
use mtool_interactive_model::CompletionMeta;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::oneshot;

use crate::complete::{CompleteRead, CompletionArgs};

use super::InteractiveWindow;

struct Context {
    args: CompletionArgs,
    tx: oneshot::Sender<String>,
}

impl Context {
    fn new(args: CompletionArgs, tx: oneshot::Sender<String>) -> Self {
        Self { args, tx }
    }
}

pub struct Completion {
    win: Res<InteractiveWindow>,
    ctx: Mutex<Option<Context>>,
}

impl Completion {
    pub async fn new(
        app: Res<AppHandle<tauri::Wry>>,
        win: Res<InteractiveWindow>,
    ) -> Result<Res<crate::Completion>, anyhow::Error> {
        let self_ = Arc::new(Self {
            win,
            ctx: Mutex::new(None),
        });

        app.manage(self_.clone());

        Ok(Res::new(crate::Completion(self_)))
    }
}

#[async_trait]
impl CompleteRead for Completion {
    async fn complete_read(&self, args: CompletionArgs) -> Result<String, anyhow::Error> {
        let id = args.meta.id.clone();
        let (tx, rx) = oneshot::channel();
        {
            let mut ctx = self.ctx.lock().await;
            *ctx = Some(Context::new(args, tx));
        }

        self.win.show().context("show completion window")?;

        self.win.emit("route", format!("/completion/{}", id))?;

        let result = rx.await?;

        self.win.hide()?;

        Ok(result)
    }
}

#[command]
async fn completion_meta(
    c: State<'_, Arc<Completion>>,
) -> Result<CompletionMeta, serde_error::Error> {
    let ctx = c.ctx.lock().await;
    ctx.as_ref()
        .map(|v| v.args.meta.clone())
        .ok_or(serde_error::Error::new(&*anyhow::anyhow!(
            "Completion context is not exist"
        )))
}

#[command]
async fn complete_read(
    completed: String,
    c: State<'_, Arc<Completion>>,
) -> Result<Vec<String>, serde_error::Error> {
    let ctx = c.ctx.lock().await;

    let ctx = ctx
        .as_ref()
        .ok_or(serde_error::Error::new(&*anyhow::anyhow!(
            "Completion context is not exist"
        )))?;

    ctx.args
        .complete(completed)
        .await
        .map_err(|e| serde_error::Error::new(&*e))
}

#[command]
async fn complete_exit(
    completed: String,
    c: State<'_, Arc<Completion>>,
) -> Result<(), serde_error::Error> {
    log::debug!("complete_exit");
    let mut ctx = c.ctx.lock().await;
    let ctx = ctx.take().ok_or(serde_error::Error::new(&*anyhow::anyhow!(
        "Completion context is not exist"
    )))?;
    ctx.tx.send(completed).unwrap();
    Ok(())
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    Builder::new("interactive::completion")
        .invoke_handler(tauri::generate_handler![
            complete_read,
            complete_exit,
            completion_meta
        ])
        .build()
}
