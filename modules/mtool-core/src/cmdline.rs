use std::{mem, ops::DerefMut, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use clap::{command, Command};
use mapp::{AppContext, AppModule, Injector, Res};
use tokio::sync::Mutex;

use super::StartupStage;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct(Cmdline::new).await;

        ctx.schedule()
            .add_task(StartupStage::PostStartup, parse_cmdline)
            .await;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Cmdline {
    inner: Arc<Mutex<Command>>,
}

impl Cmdline {
    pub async fn new() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self::default()))
    }

    pub async fn setup<F>(&self, f: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(Command) -> Result<Command, anyhow::Error>,
    {
        let cmd = self.take().await;
        self.replace(f(cmd)?).await;
        Ok(())
    }

    async fn take(&self) -> Command {
        self.replace(command!()).await
    }

    async fn replace(&self, cmd: Command) -> Command {
        mem::replace(self.inner.lock().await.deref_mut(), cmd)
    }
}

impl Default for Cmdline {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(command!())),
        }
    }
}

async fn parse_cmdline(injector: Injector) -> Result<(), anyhow::Error> {
    let cmdline = injector
        .remove::<Res<Cmdline>>()
        .await
        .take()
        .context("Failed to get Cmdline")?;

    injector
        .insert(Res::new(cmdline.take().await.get_matches()))
        .await;

    Ok(())
}
