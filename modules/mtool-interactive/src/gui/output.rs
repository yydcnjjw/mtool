use std::sync::{Arc, RwLock};

use anyhow::Context as _;
use async_trait::async_trait;
use mapp::provider::Res;
use mtool_interactive_model::OutputContent;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

use crate::{output::Output, utils::rand_string};

use super::InteractiveWindow;

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
    win: Res<InteractiveWindow>,
    ctx: RwLock<Context>,
}

impl OutputDevice {
    pub async fn new(
        app: Res<AppHandle<tauri::Wry>>,
        win: Res<InteractiveWindow>,
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
    async fn show_plain(&self, s: &str) -> Result<(), anyhow::Error> {
        self.win.show().context("show output window")?;

        self.win
            .emit("route", format!("/output/{}", rand_string()))?;

        let mut ctx = self.ctx.write().unwrap();
        ctx.content = OutputContent::Plain(s.to_string());

        Ok(())
    }
}

#[command]
async fn current_content(
    c: State<'_, Arc<OutputDevice>>,
) -> Result<OutputContent, serde_error::Error> {
    let ctx = c.ctx.read().unwrap();
    Ok(ctx.content.clone())
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    Builder::new("output")
        .invoke_handler(tauri::generate_handler![current_content])
        .build()
}
