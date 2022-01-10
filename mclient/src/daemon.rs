use std::{fmt::Debug, sync::Arc};

use cmder_mod::Command;
use mrpc::async_trait;
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Debug)]
pub struct DaemonCmd {
    serve: JoinHandle<()>,
}

impl DaemonCmd {
    pub fn new(serve: JoinHandle<()>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { serve }))
    }
}

#[async_trait]
impl Command for DaemonCmd {
    async fn exec(&mut self, _args: Vec<String>) {
        if let Err(e) = (&mut self.serve).await {
            log::error!("{:?}", e);
        }
    }
}
