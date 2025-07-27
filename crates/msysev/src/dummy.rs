use mapp::{anyhow, tokio::sync::mpsc};

use crate::event::Event;

pub struct PlatformEventLoop {}

impl PlatformEventLoop {
    pub fn new(_sender: mpsc::UnboundedSender<Event>) -> Result<Self, anyhow::Error> {
        Ok(Self {})
    }

    pub fn exit_signal(&self) -> ExitSignal {
        ExitSignal
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub struct ExitSignal;

impl ExitSignal {
    pub fn exit(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
