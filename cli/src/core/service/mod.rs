pub mod sysev;

use async_trait::async_trait;

#[async_trait]
pub trait Service {
    async fn run_loop(&mut self);
}
