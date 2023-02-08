use serde::{Deserialize, Serialize};

use self::{egress::EgressConfig, ingress::IngressConfig, routing::RoutingConfig};

pub mod egress;
pub mod ingress;
pub mod routing;
pub mod transport;

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub ingress: Vec<IngressConfig>,
    pub egress: Vec<EgressConfig>,
    pub routing: RoutingConfig,
}
