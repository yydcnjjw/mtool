use std::sync::Arc;

use anyhow::Context as _;
use async_trait::async_trait;
use mapp::provider::Res;
use mtool_interactive_model::{CompletionExit, CompletionItem, CompletionMeta};
use mtool_wgui::MtoolWindow;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::{oneshot, Mutex};
use yew::ServerRenderer;

use crate::{
    complete::{CompleteRead, CompletionArgs},
    Complete, CompleteItem,
};

#[async_trait]
trait WGuiComplete {
    async fn complete_meta(&self) -> CompletionMeta;
    async fn complete(&self, completed: &str) -> Result<Vec<CompletionItem>, anyhow::Error>;
    async fn complete_exit(&self, v: CompletionExit) -> Result<(), anyhow::Error>;
}

struct CompletionContext<T>
where
    T: CompleteItem,
{
    completion_args: CompletionArgs<T>,
    tx: Mutex<Option<oneshot::Sender<T>>>,
    items: Mutex<Vec<T>>,
}

impl<T> CompletionContext<T>
where
    T: CompleteItem,
{
    fn new(completion_args: CompletionArgs<T>, tx: oneshot::Sender<T>) -> Self {
        Self {
            completion_args,
            tx: Mutex::new(Some(tx)),
            items: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl<T> WGuiComplete for CompletionContext<T>
where
    T: CompleteItem,
{
    async fn complete_meta(&self) -> CompletionMeta {
        self.completion_args.completion_meta().clone()
    }

    async fn complete(&self, completed: &str) -> Result<Vec<CompletionItem>, anyhow::Error> {
        let mut items = self.items.lock().await;
        *items = self.completion_args.complete(completed).await?;

        let mut vec = Vec::new();
        for (i, v) in items.iter().cloned().enumerate() {
            vec.push(CompletionItem {
                id: i,
                view: ServerRenderer::<T::WGuiView>::with_props(|| v.into())
                    .render()
                    .await,
            })
        }

        Ok(vec)
    }

    async fn complete_exit(&self, v: CompletionExit) -> Result<(), anyhow::Error> {
        if let Some(tx) = self.tx.lock().await.take() {
            match v {
                CompletionExit::Id(id) => {
                    let items = self.items.lock().await;
                    let _ = tx.send(items[id].clone());
                }
                _ => {}
            }
        } else {
            anyhow::bail!("complete_exit multiple called, complete is finished");
        }
        Ok(())
    }
}

pub struct Completion {
    win: Res<MtoolWindow>,
    complete: Mutex<Option<Box<dyn WGuiComplete + Send + Sync>>>,
}

impl Completion {
    pub async fn construct(
        app: Res<AppHandle<tauri::Wry>>,
        win: Res<MtoolWindow>,
    ) -> Result<Res<crate::Completion>, anyhow::Error> {
        let self_ = Arc::new(Self {
            win,
            complete: Mutex::new(None),
        });

        app.manage(self_.clone());

        Ok(Res::new(crate::Completion::WGui(self_)))
    }

    async fn set_context<T>(&self, ctx: CompletionContext<T>)
    where
        T: CompleteItem,
    {
        *self.complete.lock().await = Some(Box::new(ctx));
    }
}

#[async_trait]
impl CompleteRead for Completion {
    async fn complete_read<T>(&self, args: CompletionArgs<T>) -> Result<T, anyhow::Error>
    where
        T: CompleteItem,
    {
        let id = args.completion_meta().id.clone();
        let need_hide_window = args.need_hide_window();

        let (tx, rx) = oneshot::channel();
        self.set_context(CompletionContext::new(args, tx)).await;

        self.win.show().context("show completion window")?;

        self.win
            .emit("route", format!("/interactive/completion/{}", id))?;

        let result = match rx.await {
            Err(_) => {
                anyhow::bail!("complete read canceled")
            }
            Ok(v) => v,
        };

        if need_hide_window {
            self.win.hide()?;
        }

        Ok(result)
    }
}

#[command]
async fn completion_meta(
    c: State<'_, Arc<Completion>>,
) -> Result<CompletionMeta, serde_error::Error> {
    let c = c.complete.lock().await;
    Ok(c.as_ref()
        .ok_or(serde_error::Error::new(&*anyhow::anyhow!(
            "completion context is not exist"
        )))?
        .complete_meta()
        .await)
}

#[command]
async fn complete(
    completed: String,
    c: State<'_, Arc<Completion>>,
) -> Result<Vec<CompletionItem>, serde_error::Error> {
    let c = c.complete.lock().await;
    c.as_ref()
        .ok_or(serde_error::Error::new(&*anyhow::anyhow!(
            "completion context is not exist"
        )))?
        .complete(&completed)
        .await
        .map_err(|e| serde_error::Error::new(&*e))
}

#[command]
async fn complete_exit(
    v: CompletionExit,
    c: State<'_, Arc<Completion>>,
) -> Result<(), serde_error::Error> {
    let c = c.complete.lock().await;
    c.as_ref()
        .ok_or(serde_error::Error::new(&*anyhow::anyhow!(
            "completion context is not exist"
        )))?
        .complete_exit(v)
        .await
        .map_err(|e| serde_error::Error::new(&*e))
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    Builder::new("interactive::completion")
        .invoke_handler(tauri::generate_handler![
            complete,
            complete_exit,
            completion_meta
        ])
        .build()
}
