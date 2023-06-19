pub mod egress;
pub mod ingress;
pub mod routing;
pub mod transport;
pub mod protos;

use serde::{Deserialize, Serialize};

use self::{egress::EgressConfig, ingress::IngressConfig, routing::RoutingConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub ingress: Vec<IngressConfig>,
    pub egress: Vec<EgressConfig>,
    pub routing: RoutingConfig,
}
