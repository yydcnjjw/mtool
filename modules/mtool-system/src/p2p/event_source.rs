use std::ops::Deref;

use mapp::{
    anyhow,
    prelude::*,
    serde::{Deserialize, Serialize},
};
use mtool_p2p::{self as p2p, gossipsub::IdentTopic, Subject};

use crate::SystemEvent;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct RemoteSystemEvent {
    pub source: String,
    pub event: SystemEvent,
}

pub struct RemoteSystemEventSource {
    subject: Subject<RemoteSystemEvent>,
}

impl RemoteSystemEventSource {
    pub(crate) async fn construct(peer: Res<p2p::Peer>) -> Result<Res<Self>, anyhow::Error> {
        let subject = peer.subscribe(&IdentTopic::new("SYSTEM_EVENT")).await?;
        Ok(Res::new(Self { subject }))
    }
}

impl Deref for RemoteSystemEventSource {
    type Target = Subject<RemoteSystemEvent>;

    fn deref(&self) -> &Self::Target {
        &self.subject
    }
}
