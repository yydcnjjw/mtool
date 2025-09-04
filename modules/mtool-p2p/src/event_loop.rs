use std::collections::{HashMap, HashSet};

use libp2p::{
    core::transport::ListenerId,
    gossipsub::{self, Message, TopicHash},
    identify, kad,
    swarm::{Swarm, SwarmEvent},
};
use mapp::{
    futures::StreamExt,
    itertools::Itertools,
    tokio::{
        select,
        sync::{broadcast, mpsc},
    },
    tracing::{debug, info, warn},
};

use crate::{
    network::{self, Behaviour, BehaviourEvent, Command, Event},
    BootNode, Config, GossipsubStats, Stats,
};

pub struct EventLoop {
    cfg: Config,

    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::UnboundedReceiver<Command>,
    event_sender: broadcast::Sender<Event>,

    listeners: HashSet<ListenerId>,

    topic_sources: HashMap<TopicHash, broadcast::Sender<Message>>,
}

impl EventLoop {
    pub fn new(
        cfg: Config,
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::UnboundedReceiver<Command>,
        event_sender: broadcast::Sender<Event>,
    ) -> Self {
        Self {
            cfg,
            swarm,
            command_receiver,
            event_sender,

            listeners: HashSet::new(),

            topic_sources: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        loop {
            select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.recv() => match command {
                    Some(c) => if !self.handle_command(c).await {
                        break
                    },
                    None => break,
                },
            }
        }

        info!("p2p event loop exited!");
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                debug!(?listener_id, ?address, "NewListenAddr");
                self.listeners.insert(listener_id);
            }
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => {
                debug!(?listener_id, ?address, "ExpiredListenAddr");
                self.listeners.remove(&listener_id);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                ..
            } => {
                info!(?peer_id, ?connection_id, ?endpoint, "ConnectionEstablished");
                _ = self
                    .event_sender
                    .send(Event::ConnectionEstablished { peer_id, endpoint });
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                ..
            } => {
                warn!(?peer_id, ?connection_id, ?endpoint, "ConnectionClosed");
            }

            SwarmEvent::NewExternalAddrCandidate { address } => {
                debug!(?address, "NewExternalAddrCandidate");
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                connection_id,
                peer_id,
                info,
            })) => {
                info!(?connection_id, ?peer_id, ?info, "Identify Received");
                if info.protocols.iter().any(|p| *p == kad::PROTOCOL_NAME) {
                    let kademlia = &mut self.swarm.behaviour_mut().kademlia;
                    _ = kademlia.remove_peer(&peer_id);
                    for addr in info.listen_addrs {
                        info!(?connection_id, ?peer_id, ?addr, "kad: add address");
                        _ = kademlia.add_address(&peer_id, addr);
                    }
                }

                #[cfg(not(feature = "server"))]
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .add_explicit_peer(&peer_id);
            }
            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(behavior)) => match behavior {
                gossipsub::Event::Message {
                    propagation_source,
                    message,
                    ..
                } => {
                    debug!(?propagation_source, ?message.source, ?message.topic);

                    if let Some(sender) = self.topic_sources.get(&message.topic) {
                        if let Err(e) = sender.send(message) {
                            warn!("{e:?}");
                        }
                    }
                }
                _ => {}
            },
            event => {
                debug!(?event)
            }
        }
    }

    async fn handle_command(&mut self, command: Command) -> bool {
        debug!(?command);

        match command {
            Command::Bootstrap { result } => {
                result.with(|| {
                    if let Some(BootNode { peer_id, address }) = &self.cfg.boot_node {
                        let kad = &mut self.swarm.behaviour_mut().kademlia;
                        kad.add_address(&peer_id.parse()?, address.parse()?);
                        _ = kad.bootstrap();
                    }

                    for id in &self.listeners {
                        self.swarm.remove_listener(id.clone());
                    }
                    self.listeners.clear();

                    network::listen_on(&mut self.swarm, &self.cfg)?;
                    Ok(())
                });
            }

            Command::Subscribe { topic, result } => result.with(move || {
                let source = self.topic_sources.entry(topic.hash()).or_insert_with(|| {
                    info!("subscribe -> {topic}");
                    let (tx, _) = broadcast::channel(64);
                    tx
                });

                _ = self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

                Ok(source.subscribe())
            }),

            Command::Unsubscribe { topic, result } => {
                result.with(move || Ok(self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic)))
            }

            Command::Publish {
                topic,
                data,
                result,
            } => {
                result.with(|| {
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(topic.hash(), data.clone())
                    {
                        warn!("{e:?}");
                    }
                    Ok(())
                });
            }

            Command::Stats { result } => result.with(move || {
                let network_info = self.swarm.network_info();
                let connected_peers = self.swarm.connected_peers().cloned().collect();

                let gossipsub = &self.swarm.behaviour().gossipsub;
                let topics = gossipsub.topics().cloned().collect_vec();
                let all_peers = gossipsub
                    .all_peers()
                    .map(|(peer, topics)| (peer.clone(), topics.into_iter().cloned().collect_vec()))
                    .collect_vec();

                let all_mesh_peers = gossipsub
                    .topics()
                    .map(|topic| {
                        (
                            topic.clone(),
                            gossipsub.mesh_peers(topic).cloned().collect_vec(),
                        )
                    })
                    .collect_vec();

                Ok(Stats {
                    network_info,
                    connected_peers,
                    gossipsub: GossipsubStats {
                        topics,
                        all_peers,
                        all_mesh_peers,
                    },
                })
            }),
            Command::Shutdown => {
                let peers = { self.swarm.connected_peers().cloned().collect_vec() };
                for peer_id in peers {
                    _ = self.swarm.disconnect_peer_id(peer_id);
                }
                return false;
            }
        }

        true
    }
}
