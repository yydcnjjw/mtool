use std::{fmt::Debug, marker::PhantomData, pin::Pin};

use libp2p::{
    gossipsub::{IdentTopic, Message, TopicHash},
    PeerId,
};
use mapp::{
    anyhow::{self, anyhow},
    futures::{Stream, StreamExt},
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
    tokio::sync::{broadcast, mpsc},
    tokio_stream::wrappers::BroadcastStream,
    tracing::{debug, warn},
};

use crate::{
    network::{Command, CommandResult, Event},
    Stats,
};

pub struct Peer {
    id: PeerId,
    command_sender: mpsc::UnboundedSender<Command>,
    _event_sender: broadcast::Sender<Event>,
}

impl Peer {
    pub(crate) fn new(
        id: PeerId,
        command_sender: mpsc::UnboundedSender<Command>,
        event_sender: broadcast::Sender<Event>,
    ) -> Self {
        Self {
            id,
            command_sender,
            _event_sender: event_sender,
        }
    }

    pub fn id(&self) -> &PeerId {
        &self.id
    }

    pub async fn bootstrap(&self) -> Result<(), anyhow::Error> {
        let (result, rx) = CommandResult::new();
        self.command_sender.send(Command::Bootstrap { result })?;
        rx.await?
    }

    pub async fn subscribe<T>(&self, topic: &IdentTopic) -> Result<Subject<T>, anyhow::Error> {
        let (result, rx) = CommandResult::new();
        self.command_sender.send(Command::Subscribe {
            topic: topic.clone(),
            result,
        })?;

        Ok(Subject::new(topic, self, rx.await??))
    }

    pub async fn unsubscribe(&self, topic: &IdentTopic) -> Result<bool, anyhow::Error> {
        let (result, rx) = CommandResult::new();
        self.command_sender.send(Command::Unsubscribe {
            topic: topic.clone(),
            result,
        })?;
        rx.await?
    }

    pub async fn publish<T>(&self, topic: &IdentTopic, data: &T) -> Result<(), anyhow::Error>
    where
        T: Serialize + Debug,
    {
        debug!("Publishing to {topic}: {data:?}");

        let (result, rx) = CommandResult::new();
        self.command_sender.send(Command::Publish {
            topic: topic.clone(),
            data: serde_json::to_vec(data)?,
            result,
        })?;
        rx.await?
    }

    pub async fn stats(&self) -> Result<Stats, anyhow::Error> {
        let (result, rx) = CommandResult::new();
        self.command_sender.send(Command::Stats { result })?;
        rx.await?
    }

    pub(crate) fn shutdown(&self) {
        if let Err(e) = self.command_sender.send(Command::Shutdown) {
            warn!("{e:?}");
        }
    }
}

pub struct Subject<T> {
    topic: IdentTopic,
    command_sender: mpsc::UnboundedSender<Command>,
    receiver: broadcast::Receiver<Message>,
    _phantom_data: PhantomData<T>,
}

impl<T> Subject<T> {
    fn new(topic: &IdentTopic, peer: &Peer, receiver: broadcast::Receiver<Message>) -> Self {
        Self {
            topic: topic.clone(),
            command_sender: peer.command_sender.clone(),
            receiver,
            _phantom_data: Default::default(),
        }
    }
}

impl<T> Subject<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    pub fn stream(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<SubjectMessage<T>, anyhow::Error>> + Send>> {
        BroadcastStream::new(self.receiver.resubscribe())
            .map(|message| match message {
                Ok(Message {
                    source,
                    data,
                    sequence_number: _,
                    topic,
                }) => Ok(SubjectMessage {
                    source,
                    topic,
                    data: serde_json::from_slice::<T>(&data)?,
                }),
                Err(e) => Err(anyhow!("{e:?}")),
            })
            .boxed()
    }

    pub async fn publish(&self, data: &T) -> Result<(), anyhow::Error> {
        let data = serde_json::to_string(data)?;
        let topic = self.topic.clone();

        debug!("publishing to topic {topic}: {data}");

        let (result, rx) = CommandResult::new();
        self.command_sender.send(Command::Publish {
            topic,
            data: data.into_bytes(),
            result,
        })?;
        rx.await?
    }
}

pub struct SubjectMessage<T> {
    pub source: Option<PeerId>,
    pub topic: TopicHash,
    pub data: T,
}
