use async_trait::async_trait;

use crate::output::Output;

pub struct OutputDevice {}

impl OutputDevice {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Output for OutputDevice {
    async fn show_plain(&self, s: &str) -> Result<(), anyhow::Error> {
        print!("{}", s);
        Ok(())
    }
}
