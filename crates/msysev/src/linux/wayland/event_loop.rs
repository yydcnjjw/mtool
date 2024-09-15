use mapp::{anyhow, tokio::sync::mpsc};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use wayland_client::{protocol::wl_registry::WlRegistry, Connection, EventQueue};

use crate::Event;

use super::state::State;

pub struct PlatformEventLoop {
    queue: EventQueue<State>,
    _registry: WlRegistry,
    state: State,

    exit_signal: ExitSignal,
}

#[derive(Clone)]
pub struct ExitSignal(Arc<AtomicBool>);

impl ExitSignal {
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    fn exited(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn exit(self) -> Result<(), anyhow::Error> {
        self.0.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl PlatformEventLoop {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Result<Self, anyhow::Error> {
        let conn = Connection::connect_to_env()?;

        let display = conn.display();

        let queue = conn.new_event_queue();

        let qh = queue.handle();
        let registry = display.get_registry(&qh, ());

        Ok(Self {
            queue,
            _registry: registry,
            state: State::new(sender),
            exit_signal: ExitSignal::new(),
        })
    }

    pub fn exit_signal(&self) -> ExitSignal {
        self.exit_signal.clone()
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let Self {
            mut queue,
            _registry: _,
            state: mut context,
            exit_signal,
        } = self;

        while !exit_signal.exited() {
            queue.blocking_dispatch(&mut context)?;
        }

        Ok(())
    }
}
