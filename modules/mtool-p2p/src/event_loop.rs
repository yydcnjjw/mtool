use std::collections::HashMap;

use libp2p::{
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
    network::{Behaviour, BehaviourEvent, Command, CommandResult, Event},
    GossipsubStats, Stats,
};

pub struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::UnboundedReceiver<Command>,
    event_sender: broadcast::Sender<Event>,

    pending_publish: HashMap<TopicHash, Vec<(Vec<u8>, CommandResult<()>)>>,

    topic_sources: HashMap<TopicHash, broadcast::Sender<Message>>,
}

impl EventLoop {
    pub fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::UnboundedReceiver<Command>,
        event_sender: broadcast::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,

            pending_publish: HashMap::new(),

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
                info!(?listener_id, ?address);
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
                info!(?peer_id, ?connection_id, ?endpoint, "ConnectionClosed");
            }

            SwarmEvent::NewExternalAddrCandidate { address } => {
                info!(?address, "NewExternalAddrCandidate");
                self.swarm.add_external_address(address);
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                connection_id,
                peer_id,
                info,
            })) => {
                if info.protocols.iter().any(|p| *p == kad::PROTOCOL_NAME) {
                    let kademlia = &mut self.swarm.behaviour_mut().kademlia;
                    _ = kademlia.remove_peer(&peer_id);
                    for addr in info.listen_addrs {
                        info!(?connection_id, ?peer_id, ?addr, "kad: add address");
                        _ = kademlia.add_address(&peer_id, addr);
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(behavior)) => {
                info!(?behavior);
                match behavior {
                    gossipsub::Event::Message { message, .. } => {
                        if let Some(sender) = self.topic_sources.get(&message.topic) {
                            if let Err(e) = sender.send(message) {
                                warn!("{e:?}");
                            }
                        }
                    }
                    gossipsub::Event::Subscribed { peer_id: _, topic } => {
                        if let Some(items) = self.pending_publish.remove(&topic) {
                            let gossipsub = &mut self.swarm.behaviour_mut().gossipsub;
                            for (data, result) in items {
                                result.with(|| {
                                    if let Err(e) = gossipsub.publish(topic.clone(), data) {
                                        warn!("{e:?}");
                                    }
                                    Ok(())
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            event => {
                debug!(?event)
            }
        }
    }

    async fn handle_command(&mut self, command: Command) -> bool {
        debug!(?command);

        match command {
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
                let gossipsub = &mut self.swarm.behaviour_mut().gossipsub;
                let has_topic = gossipsub
                    .all_peers()
                    .find(|(_, topics)| topics.contains(&&topic.hash()))
                    .is_some();
                if has_topic {
                    result.with(|| {
                        if let Err(e) = self
                            .swarm
                            .behaviour_mut()
                            .gossipsub
                            .publish(topic.hash(), data)
                        {
                            warn!("{e:?}");
                        }
                        Ok(())
                    });
                } else {
                    let entry = self.pending_publish.entry(topic.hash()).or_default();
                    entry.push((data, result));
                }
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

                Ok(Stats {
                    network_info,
                    connected_peers,
                    gossipsub: GossipsubStats { topics, all_peers },
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
