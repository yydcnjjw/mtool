pub mod admin;
pub mod egress;
pub mod ingress;
pub mod routing;
pub mod transport;

use serde::{Deserialize, Serialize};

use self::{
    admin::AdminServerConfig, egress::EgressConfig, ingress::IngressConfig, routing::RoutingConfig,
    transport::AcceptorConfig,
};

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub ingress: Vec<IngressConfig>,
    pub egress: Vec<EgressConfig>,
    pub routing: RoutingConfig,

    pub admin: AdminServerConfig,
}
