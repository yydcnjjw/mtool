use std::sync::Arc;

use anyhow::Context as _;
use async_trait::async_trait;
use futures::future::BoxFuture;
use mapp::provider::Res;
use mtool_interactive_model::OutputContent;
use mtool_wgui::WGuiWindow;
use tauri::{
    async_runtime::RwLock,
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

use crate::{output::Output, utils::rand_string};

#[derive(Debug)]
struct Context {
    content: OutputContent,
}

impl Context {
    fn new() -> Self {
        Self {
            content: OutputContent::None,
        }
    }
}

pub struct OutputDevice {
    win: Res<WGuiWindow>,
    ctx: RwLock<Context>,
}

impl OutputDevice {
    pub async fn construct(
        app: Res<AppHandle<tauri::Wry>>,
        win: Res<WGuiWindow>,
    ) -> Result<Res<crate::OutputDevice>, anyhow::Error> {
        let self_ = Arc::new(Self {
            win,
            ctx: RwLock::new(Context::new()),
        });

        app.manage(self_.clone());

        Ok(Res::new(crate::OutputDevice(self_)))
    }
}

#[async_trait]
impl Output for OutputDevice {
    async fn output(&self, s: &str) -> Result<(), anyhow::Error> {
        self.win.show().context("show output window")?;

        self.win
            .emit("route", format!("/interactive/output/{}", rand_string()))?;

        let mut ctx = self.ctx.write().await;
        ctx.content = OutputContent::Plain(s.to_string());

        Ok(())
    }

    async fn output_future(&self, o: BoxFuture<'static, String>) -> Result<(), anyhow::Error> {
        self.win.show().context("show output window")?;

        self.win
            .emit("route", format!("/interactive/output/{}", rand_string()))?;

        let mut ctx = self.ctx.write().await;
        ctx.content = OutputContent::Plain(o.await);

        Ok(())
    }
}

#[command]
async fn current_content(
    c: State<'_, Arc<OutputDevice>>,
) -> Result<OutputContent, serde_error::Error> {
    let ctx = c.ctx.read().await;
    Ok(ctx.content.clone())
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    Builder::new("interactive::output")
        .invoke_handler(tauri::generate_handler![current_content])
        .build()
}
