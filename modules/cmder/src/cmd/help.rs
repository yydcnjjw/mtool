use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{Command, ServiceClient as Cmder, ServicePoster};

pub struct Help<CmderPoster> {
    cmder: Cmder<CmderPoster>,
}

impl<CmderPoster> std::fmt::Debug for Help<CmderPoster> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HelpCmd").finish()
    }
}

impl<CmderPoster> Help<CmderPoster> {
    pub fn new(cmder: Cmder<CmderPoster>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { cmder }))
    }
}

#[async_trait]
impl<CmderPoster> Command for Help<CmderPoster>
where
    CmderPoster: Send + Sync + ServicePoster,
{
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
