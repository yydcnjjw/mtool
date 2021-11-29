mod server;

use async_trait::async_trait;

use self::server::Server;

#[async_trait]
pub trait Service {
    async fn run_loop(&mut self);
}
