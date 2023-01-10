use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;

#[async_trait]
pub trait Output {
    async fn show_plain(&self, s: &str) -> Result<(), anyhow::Error>;
    // async fn show_markdown(&self, s: &str) -> Result<(), anyhow::Error> {}
    // async fn show_html(&self) -> Result<(), anyhow::Error> {}
}

pub type SharedOutput = Arc<dyn Output + Send + Sync>;

pub struct OutputDevice(pub SharedOutput);

impl Deref for OutputDevice {
    type Target = SharedOutput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
