use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::{
    app::QuitApp,
    core::evbus::{post_result, Event, Receiver, ResponsiveEvent, Sender},
};

use super::{Command, Output};

use anyhow::Context;
use tokio::sync::Mutex;

type Cmd = Arc<Mutex<dyn Command + Send + Sync>>;

pub struct AddCommand {
    func: String,
    cmd: Cmd,
}

impl AddCommand {
    pub async fn post<Cmd>(sender: &Sender, func: String, cmd: Cmd) -> anyhow::Result<()>
    where
        Cmd: 'static + Command + Send + Sync,
    {
        post_result::<AddCommand, ()>(
            sender,
            AddCommand {
                func,
                cmd: Arc::new(Mutex::new(cmd)),
            },
        )
        .await
    }
}

pub struct RemoveCommand {
    func: String,
}

impl RemoveCommand {
    #[allow(dead_code)]
    pub async fn post(sender: &Sender, func: String) -> anyhow::Result<()> {
        post_result::<RemoveCommand, ()>(sender, RemoveCommand { func }).await
    }
}

pub struct ExecCommand {
    func: String,
    args: Vec<String>,
}

impl ExecCommand {
    pub async fn post_output(
        sender: &Sender,
        func: String,
        args: Vec<String>,
    ) -> anyhow::Result<Output> {
        post_result::<ExecCommand, anyhow::Result<Output>>(sender, ExecCommand { func, args })
            .await?
    }

    pub async fn post_none(sender: &Sender, func: String, args: Vec<String>) -> anyhow::Result<()> {
        let o = ExecCommand::post_output(sender, func.clone(), args).await?;
        match o {
            Output::None => Ok(()),
            _ => Err(anyhow::anyhow!(format!("{} output is None", func))),
        }
    }

    pub async fn post_any<T>(
        sender: &Sender,
        func: String,
        args: Vec<String>,
    ) -> anyhow::Result<Arc<T>>
    where
        T: 'static + Send + Sync,
    {
        let o = ExecCommand::post_output(sender, func.clone(), args).await?;
        match o {
            Output::Any(o) => o.downcast::<T>().map_err(|_| {
                anyhow::anyhow!(format!("Try cast to {:?} failed", TypeId::of::<T>()))
            }),
            _ => Err(anyhow::anyhow!(format!("{} output is Any", func))),
        }
    }
}

pub struct Commander {
    cmds: HashMap<String, Cmd>,
}

impl Commander {
    fn new() -> Self {
        Self {
            cmds: HashMap::new(),
        }
    }

    async fn exec(&self, name: &String, args: &[String]) -> anyhow::Result<Output> {
        let cmd = self
            .get(name)
            .with_context(|| format!("Command `{}` not found", name))?;
        cmd.lock().await.exec(args.to_vec()).await
    }

    #[allow(dead_code)]
    fn list_command_name(&self) -> Vec<&String> {
        self.cmds.keys().collect::<_>()
    }

    fn get(&self, name: &String) -> Option<&Cmd> {
        self.cmds.get(name)
    }

    fn insert(&mut self, name: String, cmd: Cmd) {
        self.cmds.insert(name, cmd);
    }

    fn remove(&mut self, name: &String) {
        self.cmds.remove(name);
    }

    pub async fn run_loop(mut rx: Receiver) {
        let mut cmder = Commander::new();

        while let Ok(e) = rx.recv().await {
            if let Some(e) = e.downcast_ref::<ResponsiveEvent<AddCommand, ()>>() {
                cmder.insert(e.func.clone(), e.cmd.clone());
                e.result(());
            } else if let Some(e) = e.downcast_ref::<ResponsiveEvent<RemoveCommand, ()>>() {
                cmder.remove(&e.func);
                e.result(());
            } else if let Some(e) =
                e.downcast_ref::<ResponsiveEvent<ExecCommand, anyhow::Result<Output>>>()
            {
                e.result(cmder.exec(&e.func, &e.args).await);
            } else if let Some(_) = e.downcast_ref::<Event<QuitApp>>() {
                break;
            }
        }
    }
}
