use libp2p::{
    core::ConnectedPoint,
    gossipsub::{self, IdentTopic, Message},
    identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    multiaddr::Protocol,
    swarm::NetworkBehaviour,
    Multiaddr, PeerId,
};
use mapp::{
    anyhow,
    tokio::sync::{broadcast, mpsc, oneshot},
    tracing::{info, warn},
};
use std::{any::type_name_of_val, io, net::Ipv4Addr, time::Duration};

use crate::{BootNode, Config, EventLoop, Peer, Stats};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}

#[derive(Debug)]
pub enum Command {
    Subscribe {
        topic: IdentTopic,
        result: CommandResult<broadcast::Receiver<Message>>,
    },
    Unsubscribe {
        topic: IdentTopic,
        result: CommandResult<bool>,
    },
    Publish {
        topic: IdentTopic,
        data: Vec<u8>,
        result: CommandResult<()>,
    },
    Stats {
        result: CommandResult<Stats>,
    },
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum Event {
    ConnectionEstablished {
        peer_id: PeerId,
        endpoint: ConnectedPoint,
    },
}

#[derive(Debug)]
pub struct CommandResult<T>(oneshot::Sender<Result<T, anyhow::Error>>);

impl<T> CommandResult<T> {
    pub fn new() -> (Self, oneshot::Receiver<Result<T, anyhow::Error>>) {
        let (tx, rx) = oneshot::channel();
        (Self(tx), rx)
    }

    pub fn with<F>(self, f: F)
    where
        F: FnOnce() -> Result<T, anyhow::Error>,
    {
        if let Err(e) = self.0.send(f()) {
            warn!("Failed to send {}", type_name_of_val(&e));
        }
    }
}

pub fn new(cfg: Config) -> Result<(Peer, EventLoop), anyhow::Error> {
    let mut swarm = match &cfg.peer_id {
        Some(peer_id) => libp2p::SwarmBuilder::with_existing_identity(Keypair::ed25519_from_bytes(
            base64::decode(peer_id)?,
        )?),
        None => libp2p::SwarmBuilder::with_new_identity(),
    }
    .with_tokio()
    .with_quic()
    .with_behaviour(|key| {
        let mut kademlia = kad::Behaviour::new(
            key.public().to_peer_id(),
            MemoryStore::new(key.public().to_peer_id()),
        );

        if let Some(BootNode { peer_id, address }) = &cfg.boot_node {
            info!(?peer_id, ?address, "add boot node");
            kademlia.add_address(&peer_id.parse()?, address.parse()?);
        }

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .check_explicit_peers_ticks(15)
            .build()
            .map_err(io::Error::other)?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )?;

        Ok(Behaviour {
            kademlia,
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                key.public(),
            )),
            gossipsub,
        })
    })?
    .build();

    swarm.listen_on(
        Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Udp(cfg.listen_port.unwrap_or(0)))
            .with(Protocol::QuicV1),
    )?;

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let (event_sender, _) = broadcast::channel(64);

    Ok((
        Peer::new(
            swarm.local_peer_id().clone(),
            command_sender,
            event_sender.clone(),
        ),
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}
