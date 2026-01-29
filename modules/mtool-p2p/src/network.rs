use base64::prelude::*;
use libp2p::{
    core::ConnectedPoint,
    gossipsub::{self, IdentTopic, Message},
    identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    multiaddr::Protocol,
    swarm::NetworkBehaviour,
    Multiaddr, PeerId, Swarm,
};
use mapp::{
    anyhow,
    tokio::sync::{broadcast, mpsc, oneshot},
    tracing::{debug, info, warn},
};
use std::{any::type_name_of_val, fmt, io, time::Duration};

use crate::{BootNode, Config, EventLoop, Peer, Stats};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}

pub enum Command {
    Bootstrap {
        result: CommandResult<()>,
    },
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

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bootstrap { .. } => f.debug_struct("Bootstrap").finish(),
            Self::Subscribe { topic, .. } => {
                f.debug_struct("Subscribe").field("topic", topic).finish()
            }
            Self::Unsubscribe { topic, .. } => {
                f.debug_struct("Unsubscribe").field("topic", topic).finish()
            }
            Self::Publish { topic, .. } => f.debug_struct("Publish").field("topic", topic).finish(),
            Self::Stats { .. } => f.debug_struct("Stats").finish(),
            Self::Shutdown => write!(f, "Shutdown"),
        }
    }
}

#[allow(unused)]
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
            BASE64_STANDARD.decode(peer_id)?,
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
            .heartbeat_interval(Duration::from_secs(5))
            .check_explicit_peers_ticks(3)
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

    listen_on(&mut swarm, &cfg)?;

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
        EventLoop::new(cfg, swarm, command_receiver, event_sender),
    ))
}

pub(crate) fn listen_on(swarm: &mut Swarm<Behaviour>, cfg: &Config) -> Result<(), anyhow::Error> {
    for listen_addr in if_addrs::get_if_addrs()?
        .iter()
        .inspect(|iface| debug!(?iface))
        .filter_map(|iface| {
            if cfg.listen_iface_name_list.contains(&iface.name)
                || iface
                    .index
                    .is_some_and(|index| cfg.listen_iface_index_list.contains(&index))
            {
                Some(iface.addr.ip())
            } else {
                None
            }
        })
        .chain(cfg.listen_addr_list.iter().cloned())
    {
        info!("listen on {}", listen_addr);
        swarm.listen_on(
            Multiaddr::empty()
                .with(Protocol::from(listen_addr))
                .with(Protocol::Udp(cfg.listen_port.unwrap_or(0)))
                .with(Protocol::QuicV1),
        )?;
    }

    Ok(())
}
