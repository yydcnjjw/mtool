mod agenda_service;

pub use agenda_service::AgendaService;

use async_trait::async_trait;

#[async_trait]
pub trait Service {
    async fn run(&mut self);
}
