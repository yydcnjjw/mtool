use crate::{
    app::App,
    core::{
        evbus::{Event, Sender},
        service::Service,
    },
};

struct SysEventService {
    sender: Sender,
}

impl SysEventService {
    fn new(sender: Sender) -> Self {
        Self { sender }
    }
}

impl Event for sysev::Event {
    fn type_id() -> u32 {
        0
    }
}

impl Service for SysEventService {
    async fn run_loop(&mut self) {
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            sysev::run_loop(|e| sender.send(Box::new(e)));
        })
    }
}

pub async fn module_load(app: &mut App) -> anyhow::Result<()> {
    SysEventService::new(app.evbus.sender());
    Ok(())
}


#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;

    use super::*;

    #[test]
    fn test_sysev() {
    }
}
