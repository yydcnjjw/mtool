use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{CmderCli, Command};

pub struct Help {
    cmder: CmderCli,
}

impl std::fmt::Debug for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HelpCmd").finish()
    }
}

impl Help {
    pub fn new(cmder: CmderCli) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { cmder }))
    }
}

#[async_trait]
impl Command for Help {
    async fn exec(&mut self, _: Vec<String>) {
        match self.cmder.list().await {
            Ok(list) => list.iter().for_each(|cmd| {
                println!("{}", cmd);
            }),
            Err(e) => {
                log::error!("{:?}", e);
            }
        };
    }
}
