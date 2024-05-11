use tokio::sync::mpsc;

use crate::Event;

use super::wayland;

pub enum PlatformEventLoop {
    Wayland(wayland::PlatformEventLoop),
}

pub enum ExitSignal {
    Wayland(wayland::ExitSignal),
}

impl ExitSignal {
    pub fn exit(self) -> Result<(), anyhow::Error> {
        match self {
            ExitSignal::Wayland(inner) => inner.exit(),
        }
    }
}

impl PlatformEventLoop {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Result<Self, anyhow::Error> {
        Ok(Self::Wayland(wayland::PlatformEventLoop::new(sender)?))
    }

    pub fn exit_signal(&self) -> ExitSignal {
        match &self {
            PlatformEventLoop::Wayland(inner) => ExitSignal::Wayland(inner.exit_signal()),
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        match self {
            PlatformEventLoop::Wayland(inner) => inner.run(),
        }
    }
}
