use std::sync::{mpsc, Arc};

#[cfg(target_os = "android")]
use dioxus_desktop::wry::prelude::*;
use dioxus_desktop::{
    winit::event_loop::{ActiveEventLoop, EventLoopProxy},
    UserWindowEvent,
};

#[cfg(target_os = "android")]
use mapp::android_activity::AndroidApp;
use mapp::{
    anyhow::{self, Context},
    prelude::*,
    tokio::sync::oneshot,
    tracing::{debug, warn},
};

use crate::message::Message;

#[derive(Clone)]
pub struct DioxusContext {
    event_loop: EventLoopProxy<UserWindowEvent>,
    #[cfg(target_os = "android")]
    android_app: AndroidApp,

    message_sender: mpsc::Sender<Message>,
}

pub struct DioxusEventLoopContext {
    message_receiver: mpsc::Receiver<Message>,
}

impl DioxusContext {
    pub fn new(
        event_loop: EventLoopProxy<UserWindowEvent>,
        #[cfg(target_os = "android")] android_app: AndroidApp,
    ) -> (Self, DioxusEventLoopContext) {
        let (message_sender, message_receiver) = mpsc::channel();
        (
            Self {
                event_loop,
                #[cfg(target_os = "android")]
                android_app,
                message_sender,
            },
            DioxusEventLoopContext { message_receiver },
        )
    }

    pub fn event_loop(&self) -> EventLoopProxy<UserWindowEvent> {
        self.event_loop.clone()
    }

    pub async fn run_on_main_thread<F>(&self, task: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(DioxusContext) -> Result<(), anyhow::Error> + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let this = self.clone();

        self.message_sender
            .send(Message::Task((Box::new(|| task(this)), tx)))
            .context("send new task")?;
        self.event_loop.send_event(UserWindowEvent::WakeUp)?;
        rx.await
            .context("recv task result")?
            .map_err(|e| anyhow::anyhow!("task failed: {}", e))
    }

    #[cfg(target_os = "android")]
    pub fn android_app(&self) -> AndroidApp {
        self.android_app.clone()
    }

    #[cfg(target_os = "android")]
    pub fn jvm(&self) -> Result<jni::JavaVM, anyhow::Error> {
        unsafe { Ok(jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _)?) }
    }
}

impl DioxusEventLoopContext {
    pub(crate) fn pool_events(&self, _event_loop: &ActiveEventLoop) {
        while let Ok(msg) = self.message_receiver.try_recv() {
            debug!("handle {:?}", msg);
            match msg {
                Message::Task((task, sender)) => {
                    if let Err(e) = sender.send(task()) {
                        warn!("{:?}", e);
                    }
                }
            }
        }
    }
}
