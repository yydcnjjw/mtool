use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast, RwLock};

use crate::{command::AsyncCommand, Service, ServiceResponse, ServiceRequest};
use keybinding_mod::ServiceClient as KbCli;

pub struct Cmder {
    cmds: Arc<RwLock<HashMap<String, AsyncCommand>>>,
}

impl Cmder {
    pub async fn new(kbcli: KbCli) -> anyhow::Result<Arc<Self>> {
        let rx = kbcli.subscribe().await?;
        let self_ = Arc::new(Self {
            cmds: Arc::new(RwLock::new(HashMap::new())),
        });

        tokio::spawn(Self::run_loop(self_.clone(), rx));

        Ok(self_)
    }

    async fn run_loop(self: Arc<Self>, mut rx: broadcast::Receiver<String>) {
        while let Ok(cmd) = rx.recv().await {
            Self::exec(self.clone(), cmd, Vec::new()).await;
        }
    }
}

#[mrpc::service]
impl Service for Cmder {
    async fn add(self: Arc<Self>, name: String, cmd: AsyncCommand) {
        self.cmds.write().await.insert(name, cmd);
    }

    async fn remove(self: Arc<Self>, name: String) {
        self.cmds.write().await.remove(&name);
    }

    async fn list(self: Arc<Self>) -> Vec<String> {
        self.cmds.read().await.keys().cloned().collect()
    }

    async fn exec(self: Arc<Self>, name: String, args: Vec<String>) {
        match self.cmds.read().await.get(&name) {
            Some(cmd) => {
                cmd.lock().await.exec(args).await;
            }
            None => {
                log::warn!("Command {} not found", name);
            }
        }
    }
}
