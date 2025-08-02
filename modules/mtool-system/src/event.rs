use mapp::{anyhow, prelude::*, tokio::sync::broadcast};

use crate::platform;

#[derive(Clone, Debug)]
pub enum SystenEvent {
    NotificationPosted(Notification),
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub package_name: String,
}

pub struct SystemEventSource {
    pub(crate) inner: platform::SystemEventSource,
}

impl SystemEventSource {
    pub async fn construct(injector: Injector) -> Result<Res<SystemEventSource>, anyhow::Error> {
        Ok(Res::new(SystemEventSource {
            inner: inject_once(&injector, platform::SystemEventSource::new).await??,
        }))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystenEvent> {
        self.inner.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystenEvent> {
        self.inner.sender()
    }    
}
