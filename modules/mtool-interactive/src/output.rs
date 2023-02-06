use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use futures::future::BoxFuture;

#[async_trait]

pub trait Output: Sync {
    async fn output(&self, _: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn output_future(&self, _: BoxFuture<'static, String>) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub type SharedOutput = Arc<dyn Output + Send + Sync>;

pub struct OutputDevice(pub SharedOutput);

impl Deref for OutputDevice {
    type Target = SharedOutput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
