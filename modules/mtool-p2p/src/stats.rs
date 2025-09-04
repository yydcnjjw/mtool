use libp2p::{gossipsub::TopicHash, swarm::NetworkInfo, PeerId};

#[derive(Clone, Debug)]
pub struct Stats {
    pub network_info: NetworkInfo,
    pub connected_peers: Vec<PeerId>,
    pub gossipsub: GossipsubStats,
}

#[derive(Clone, Debug)]
pub struct GossipsubStats {
    pub topics: Vec<TopicHash>,
    pub all_peers: Vec<(PeerId, Vec<TopicHash>)>,
    pub all_mesh_peers: Vec<(TopicHash, Vec<PeerId>)>,
}
