use std::{mem, sync::Mutex};

use async_trait::async_trait;
use clap::{command, Command};
use mapp::{
    define_label,
    provider::{Injector, Res},
    AppContext, AppModule, Label,
};

use crate::AppStage;

#[derive(Default)]
pub struct Module {}

define_label!(
    pub enum CmdlineStage {
        Setup,
        Init,
        AfterInit,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(Cmdline::new);

        ctx.schedule()
            .insert_stage_vec(
                AppStage::Startup,
                vec![
                    CmdlineStage::Setup,
                    CmdlineStage::Init,
                    CmdlineStage::AfterInit,
                ],
            )
            .add_once_task(CmdlineStage::Init, parse_cmdline);
        Ok(())
    }
}

pub struct Cmdline {
    inner: Mutex<Command>,
}

impl Cmdline {
    pub async fn new() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self::default()))
    }

    pub fn setup<F>(&self, f: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(Command) -> Result<Command, anyhow::Error>,
    {
        let cmd = self.take();
        self.replace(f(cmd)?);
        Ok(())
    }

    fn take(&self) -> Command {
        self.replace(command!())
    }

    fn replace(&self, cmd: Command) -> Command {
        mem::replace(&mut self.inner.lock().unwrap(), cmd)
    }
}

impl Default for Cmdline {
    fn default() -> Self {
        Self {
            inner: Mutex::new(command!()),
        }
    }
}

async fn parse_cmdline(cmdline: Res<Cmdline>, injector: Injector) -> Result<(), anyhow::Error> {
    injector.insert(Res::new(cmdline.take().get_matches()));
    Ok(())
}
