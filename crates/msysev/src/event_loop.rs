use std::ops;

use mapp::{
    anyhow,
    tokio::{self, sync::mpsc},
    tracing::warn,
};

#[cfg(target_os = "windows")]
use crate::windows::event_loop::PlatformEventLoop;

#[cfg(target_os = "linux")]
use crate::linux::event_loop::PlatformEventLoop;

use crate::event::Event;

pub type ControlFlow = ops::ControlFlow<()>;

pub struct EventLoop {
    inner: PlatformEventLoop,
    event_sender: mpsc::UnboundedSender<Event>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventLoop {
    pub fn new() -> Result<Self, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        Ok(Self {
            inner: PlatformEventLoop::new(tx.clone())?,
            event_sender: tx,
            event_receiver: rx,
        })
    }

    pub fn exit_signal(&self) -> ExitSignal {
        ExitSignal(self.event_sender.clone())
    }

    pub async fn run<F>(self, mut event_handler: F) -> Result<(), anyhow::Error>
    where
        F: FnMut(Event) -> ControlFlow,
    {
        let Self {
            inner,
            event_sender: _,
            mut event_receiver,
        } = self;

        let exit_siganl = inner.exit_signal();

        tokio::task::spawn_blocking(|| {
            if let Err(e) = inner.run() {
                warn!("{:?}", e);
            }
        });

        while let Some(ev) = event_receiver.recv().await {
            match ev {
                Event::Exit => {
                    break;
                }
                _ => match event_handler(ev) {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => {
                        if let Err(e) = exit_siganl.exit() {
                            warn!("{:?}", e);
                        }
                        break;
                    }
                },
            }
        }
        Ok(())
    }
}

pub struct ExitSignal(mpsc::UnboundedSender<Event>);

impl ExitSignal {
    pub fn exit(&self) {
        let _ = self.0.send(Event::Exit);
    }
}
